from __future__ import annotations

import sys

from epidote.config import load_config
from epidote.pipeline import MeetingPipeline
from epidote.storage import MeetingRepository


def main() -> int:
    from PySide6.QtWidgets import QApplication

    app = QApplication(sys.argv)
    config = load_config()
    repository = MeetingRepository(config)
    pipeline = MeetingPipeline(config, repository)

    from epidote.ui.main_window import MainWindow

    window = MainWindow(config=config, repository=repository, pipeline=pipeline)
    window.show()
    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
