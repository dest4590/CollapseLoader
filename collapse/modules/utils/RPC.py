from threading import Thread
from time import sleep, time

from pypresence import Presence

from ..storage.Data import data
from ..storage.Settings import settings
from ..utils.Language import lang
from ..utils.Module import Module


class RPC(Thread, Module):
    """RPC, used to display activity in Discord"""

    def __init__(self, *args, **kwargs) -> None:
        Thread.__init__(self, **kwargs)
        Module.__init__(self)
        self.client_id = "1225803664204234772"
        self.RPC = Presence(self.client_id)
        self.default_details = lang.t("rpc.choosing")
        self.details = self.default_details
        self.start_time = time()
        self.disabled = not settings.use_option("rpc")

    def update(self):
        """Updates the activity"""

        try:
            self.RPC.update(
                state=settings.get("nickname"),
                details=self.details,
                large_image="https://i.imgur.com/ZpWg110.gif",
                buttons=[
                    {"label": "Discord", "url": "https://collapseloader.org/discord/"},
                    {
                        "label": "collapseloader.org",
                        "url": "https://collapseloader.org",
                    },
                ],
                start=self.start_time,
                large_text=lang.t("rpc.large-text").format(data.version),
            )
        except Exception:
            try:
                self.RPC.connect()
            except Exception:
                pass

    def stop(self):
        """Stops the RPC and disconnects"""

        self.disabled = True

        try:
            self.RPC.clear()
            self.RPC.close()

        except Exception:
            pass

    def run(self):
        """Starts a thread for the rpc"""

        try:
            self.RPC.connect()

        except Exception:
            pass

        while True:
            if not self.disabled:
                self.update()
            else:
                self.RPC.clear()

            sleep(5)


rpc = RPC()
rpc.daemon = True
