import os
from datetime import datetime

from ..utils.Language import lang
from .Module import Module


class LogChecker(Module):
    """Used to check game logs to detect crashes"""

    def __init__(self) -> None:
        """Initialize LogChecker and log the initialization"""
        super().__init__()

    def reason(self, msg: str) -> str:
        """Construct a reason message for the crash"""
        return lang.t("logchecker.base-reason").format(msg)

    def check_logs(self, logs: str, client) -> bool:
        """Check logs for crash messages and log appropriate errors and info"""
        self.debug(lang.t("logchecker.checking-log"))

        if "Game crashed!" in logs:
            self.error(lang.t("logchecker.game-crashed"))
            self.save_crash_log(logs, client)

            # Memory Errors
            if "java.lang.OutOfMemoryError: Java heap space" in logs:
                self.info(self.reason(lang.t("logchecker.crashes.out_of_memory")))

            # Class and Library Errors
            elif "java.lang.NoClassDefFoundError" in logs:
                self.info(self.reason(lang.t("logchecker.crashes.no_class_def")))
            elif "java.lang.UnsupportedClassVersionError" in logs:
                self.info(self.reason(lang.t("logchecker.crashes.unsupported_class")))

            # Modding Errors
            elif "LoaderExceptionModCrash" in logs:
                self.info(self.reason(lang.t("logchecker.crashes.mod_crash")))

            # World Corruption
            elif "Failed to load level" in logs:
                self.info(self.reason(lang.t("logchecker.crashes.corrupted_world")))

            # Graphics and Rendering
            elif "Pixel format not accelerated" in logs:
                self.info(self.reason(lang.t("logchecker.crashes.graphics_error")))
            elif "OpenGL error" in logs:
                self.info(self.reason(lang.t("logchecker.crashes.opengl_error")))

            # Catch-all for unknown reasons
            else:
                self.warn(lang.t("logchecker.crashes.unknown"))

            return True
        else:
            self.debug(lang.t("logchecker.no-crashes"))
            return False

    def save_crash_log(self, payload: str, client) -> None:
        """Saves the crash log to a file"""
        crash_log_dir = os.path.join("..", "crash_logs")

        self.debug(lang.t("logchecker.crash-logs.creating-dir").format(crash_log_dir))
        os.makedirs(crash_log_dir, exist_ok=True)

        crash_log_file = os.path.join(
            "..",
            f"crash_logs",
            f'{client.name}_{datetime.now().strftime("%d-%m-%Y_%H-%M-%S")}.txt',
        )

        with open(crash_log_file, "w", encoding="utf-8") as f:
            f.write(payload)

        self.info(lang.t("logchecker.crash-logs.saved").format(crash_log_file))


logchecker = LogChecker()
