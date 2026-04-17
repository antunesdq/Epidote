import AVFoundation
import CoreGraphics
import Foundation
import ScreenCaptureKit

struct RecorderConfiguration {
    let outputDirectory: URL
    let title: String

    static func parse(arguments: [String]) throws -> RecorderConfiguration {
        var outputDirectory: URL?
        var title = "Untitled meeting"
        var index = 1
        while index < arguments.count {
            let argument = arguments[index]
            switch argument {
            case "--output-dir":
                index += 1
                guard index < arguments.count else {
                    throw RecorderError.invalidArguments("--output-dir requires a value")
                }
                outputDirectory = URL(fileURLWithPath: arguments[index])
            case "--title":
                index += 1
                guard index < arguments.count else {
                    throw RecorderError.invalidArguments("--title requires a value")
                }
                title = arguments[index]
            default:
                throw RecorderError.invalidArguments("Unknown argument: \(argument)")
            }
            index += 1
        }
        guard let outputDirectory else {
            throw RecorderError.invalidArguments("Missing --output-dir")
        }
        return RecorderConfiguration(outputDirectory: outputDirectory, title: title)
    }
}

enum RecorderError: Error, LocalizedError {
    case invalidArguments(String)
    case microphonePermissionDenied
    case screenPermissionDenied
    case displayUnavailable
    case assetWriterFailed(String)

    var errorDescription: String? {
        switch self {
        case .invalidArguments(let message):
            return message
        case .microphonePermissionDenied:
            return "Microphone permission was denied."
        case .screenPermissionDenied:
            return "Screen Recording permission was denied."
        case .displayUnavailable:
            return "No capture display was available for ScreenCaptureKit."
        case .assetWriterFailed(let message):
            return "Asset writer failed: \(message)"
        }
    }
}

struct RecordingManifest: Codable {
    let title: String
    let startedAt: Date
    let stoppedAt: Date?
    let microphonePath: String
    let systemAudioPath: String
}

final class SignalTrap {
    private let semaphore = DispatchSemaphore(value: 0)
    private var sources: [DispatchSourceSignal] = []

    init(signals: [Int32]) {
        for signalValue in signals {
            signal(signalValue, SIG_IGN)
            let source = DispatchSource.makeSignalSource(
                signal: signalValue,
                queue: DispatchQueue.global(qos: .default)
            )
            source.setEventHandler { [weak self] in
                self?.semaphore.signal()
            }
            source.resume()
            sources.append(source)
        }
    }

    func wait() {
        semaphore.wait()
    }
}

final class MicrophoneRecorder {
    private let engine = AVAudioEngine()
    private var outputFile: AVAudioFile?

    func start(outputURL: URL) throws {
        let inputNode = engine.inputNode
        let format = inputNode.outputFormat(forBus: 0)
        outputFile = try AVAudioFile(forWriting: outputURL, settings: format.settings)
        inputNode.installTap(onBus: 0, bufferSize: 2048, format: format) { [weak self] buffer, _ in
            guard let outputFile = self?.outputFile else {
                return
            }
            do {
                try outputFile.write(from: buffer)
            } catch {
                fputs("Failed to write microphone buffer: \(error)\n", stderr)
            }
        }
        engine.prepare()
        try engine.start()
    }

    func stop() {
        engine.inputNode.removeTap(onBus: 0)
        engine.stop()
        outputFile = nil
    }
}

final class SystemAudioRecorder: NSObject, SCStreamOutput, SCStreamDelegate {
    private var stream: SCStream?
    private var writer: AVAssetWriter?
    private var writerInput: AVAssetWriterInput?
    private var startedWriting = false

    func start(outputURL: URL) async throws {
        let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
        guard let display = content.displays.first else {
            throw RecorderError.displayUnavailable
        }

        try prepareWriter(outputURL: outputURL)

        let filter = SCContentFilter(
            display: display,
            excludingApplications: [],
            exceptingWindows: []
        )
        let configuration = SCStreamConfiguration()
        configuration.width = display.width
        configuration.height = display.height
        configuration.minimumFrameInterval = CMTime(value: 1, timescale: 60)
        configuration.capturesAudio = true
        configuration.captureMicrophone = false
        configuration.sampleRate = 48_000
        configuration.channelCount = 2

        let stream = SCStream(filter: filter, configuration: configuration, delegate: self)
        try stream.addStreamOutput(
            self,
            type: .audio,
            sampleHandlerQueue: DispatchQueue(label: "epidote.system-audio")
        )
        try await stream.startCapture()
        self.stream = stream
    }

    func stop() async throws {
        try await stream?.stopCapture()
        writerInput?.markAsFinished()
        if let writer {
            if writer.status == .writing {
                await withCheckedContinuation { continuation in
                    writer.finishWriting {
                        continuation.resume()
                    }
                }
            } else if writer.status == .failed {
                throw RecorderError.assetWriterFailed(writer.error?.localizedDescription ?? "unknown")
            }
        }
        stream = nil
        writerInput = nil
        writer = nil
        startedWriting = false
    }

    func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of outputType: SCStreamOutputType) {
        guard outputType == .audio else {
            return
        }
        guard CMSampleBufferDataIsReady(sampleBuffer) else {
            return
        }
        guard let writer, let writerInput else {
            return
        }
        if !startedWriting {
            writer.startWriting()
            writer.startSession(atSourceTime: CMSampleBufferGetPresentationTimeStamp(sampleBuffer))
            startedWriting = true
        }
        if writerInput.isReadyForMoreMediaData {
            writerInput.append(sampleBuffer)
        }
    }

    func stream(_ stream: SCStream, didStopWithError error: Error) {
        fputs("System audio stream stopped with error: \(error)\n", stderr)
    }

    private func prepareWriter(outputURL: URL) throws {
        writer = try AVAssetWriter(outputURL: outputURL, fileType: .m4a)
        let settings: [String: Any] = [
            AVFormatIDKey: kAudioFormatMPEG4AAC,
            AVSampleRateKey: 48_000,
            AVNumberOfChannelsKey: 2,
            AVEncoderBitRateKey: 192_000,
        ]
        let writerInput = AVAssetWriterInput(mediaType: .audio, outputSettings: settings)
        writerInput.expectsMediaDataInRealTime = true
        guard let writer else {
            throw RecorderError.assetWriterFailed("Writer was not created.")
        }
        if writer.canAdd(writerInput) {
            writer.add(writerInput)
        } else {
            throw RecorderError.assetWriterFailed("Unable to add audio input.")
        }
        self.writerInput = writerInput
    }
}

final class RecorderCoordinator {
    private let configuration: RecorderConfiguration
    private let microphoneRecorder = MicrophoneRecorder()
    private let systemAudioRecorder = SystemAudioRecorder()
    private let fileManager = FileManager.default
    private let startedAt = Date()

    init(configuration: RecorderConfiguration) {
        self.configuration = configuration
    }

    var microphoneURL: URL {
        configuration.outputDirectory.appendingPathComponent("microphone.wav")
    }

    var systemAudioURL: URL {
        configuration.outputDirectory.appendingPathComponent("system_audio.m4a")
    }

    var manifestURL: URL {
        configuration.outputDirectory.appendingPathComponent("recording_manifest.json")
    }

    func start() async throws {
        try fileManager.createDirectory(at: configuration.outputDirectory, withIntermediateDirectories: true)
        try await requestPermissions()
        try microphoneRecorder.start(outputURL: microphoneURL)
        try await systemAudioRecorder.start(outputURL: systemAudioURL)
        try writeManifest(stoppedAt: nil)
    }

    func stop() async throws {
        microphoneRecorder.stop()
        try await systemAudioRecorder.stop()
        try writeManifest(stoppedAt: Date())
    }

    private func requestPermissions() async throws {
        let microphoneGranted = await withCheckedContinuation { continuation in
            AVCaptureDevice.requestAccess(for: .audio) { granted in
                continuation.resume(returning: granted)
            }
        }
        guard microphoneGranted else {
            throw RecorderError.microphonePermissionDenied
        }
        if !CGPreflightScreenCaptureAccess() {
            let granted = CGRequestScreenCaptureAccess()
            guard granted else {
                throw RecorderError.screenPermissionDenied
            }
        }
    }

    private func writeManifest(stoppedAt: Date?) throws {
        let manifest = RecordingManifest(
            title: configuration.title,
            startedAt: startedAt,
            stoppedAt: stoppedAt,
            microphonePath: microphoneURL.path,
            systemAudioPath: systemAudioURL.path
        )
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        encoder.dateEncodingStrategy = .iso8601
        let data = try encoder.encode(manifest)
        try data.write(to: manifestURL)
    }
}

@main
enum EpidoteRecorderCLI {
    static func main() async {
        do {
            let configuration = try RecorderConfiguration.parse(arguments: CommandLine.arguments)
            let coordinator = RecorderCoordinator(configuration: configuration)
            try await coordinator.start()
            print("Recording started.")
            let trap = SignalTrap(signals: [SIGINT, SIGTERM])
            trap.wait()
            try await coordinator.stop()
            print("Recording stopped.")
        } catch {
            fputs("\(error.localizedDescription)\n", stderr)
            Foundation.exit(1)
        }
    }
}
