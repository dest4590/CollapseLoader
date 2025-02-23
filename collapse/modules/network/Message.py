import datetime

from rich import print

from ...developer import SAVE_MESSAGES
from ..network.Servers import servers
from ..storage.Settings import settings
from ..utils.Language import lang
from ..utils.Module import Module
from .API import api


class Messages(Module):
    """Client for retrieving and displaying messages"""

    def __init__(self) -> None:
        """Initialize the Messages and fetch messages from the API"""
        super().__init__()
        self.shown = False
        if servers.web_server != "":
            self.messages = self.fetch_messages()
        else:
            self.messages = None
        self.types = {
            "info": f'[green]{lang.t("messages.types.info")}[/]',
            "warn": f'[yellow]{lang.t("messages.types.warning")}[/]',
            "maintenance": f'[blue]{lang.t("messages.types.maintenance")}[/]',
        }

        self.debug(lang.t("messages.fetched"))

    def fetch_messages(self):
        """Fetch messages from the API"""
        try:
            return api.get("messages")
        except Exception:
            self.error(lang.t("messages.fetch-error"))
            return None

    def show_messages(self) -> None:
        """Display unread messages"""
        if not self.messages:
            if servers.web_server != "":
                self.error(lang.t("messages.fetch-error"))
            return

        if settings.use_option("hide_messages"):
            read_message_ids = self.get_read_message_ids()
            local_tz = datetime.datetime.now(datetime.timezone.utc).astimezone().tzinfo
            current_time = datetime.datetime.now(local_tz)

            for message in self.messages.json():
                if message["id"] not in read_message_ids and not message["hidden"]:
                    self.process_message(
                        message, read_message_ids, local_tz, current_time
                    )

            self.shown = True

    def get_read_message_ids(self):
        """Get the set of read message IDs from settings"""
        read_messages = settings.get("read_messages", "Loader")
        return set(int(id) for id in read_messages.split(",")[:-1])

    def process_message(self, message, read_message_ids, local_tz, current_time):
        """Process and display a single message"""
        if SAVE_MESSAGES:
            read_message_ids.add(message["id"])
            settings.set(
                "read_messages", ",".join(map(str, read_message_ids)) + ",", "Loader"
            )

        post_time = datetime.datetime.fromisoformat(message["post_at"]).astimezone(
            local_tz
        )
        time_difference = current_time - post_time
        time_ago = self.calculate_time_ago(time_difference)

        message_type = self.types.get(message["type"], "[gray]Unknown[/]")
        print(
            f"\n{message_type} {lang.t('messages.message')} {post_time.strftime('%Y-%m-%d %H:%M:%S')} ({time_ago})\n{message['body']}\n"
        )

    @staticmethod
    def calculate_time_ago(time_difference: datetime.timedelta) -> str:
        """Calculate a human-readable time difference"""
        if time_difference < datetime.timedelta(minutes=1):
            return lang.t("messages.time-now")
        total_seconds = time_difference.total_seconds()
        if total_seconds < 3600:
            return lang.t("messages.time-minutes").format(int(total_seconds // 60))
        if total_seconds < 86400:
            return lang.t("messages.time-hours").format(int(total_seconds // 3600))
        return lang.t("messages.time-days").format(time_difference.days)


messages = Messages()
