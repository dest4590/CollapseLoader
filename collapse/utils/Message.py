import datetime

from rich import print

from ..static import SAVE_MESSAGES
from ..modules.Module import Module
from .API import api
from .Logger import logger
from .Settings import settings


class MessageClient(Module):
    """Client for retrieving and displaying messages"""

    shown = False

    def __init__(self) -> None:
        """Initialize the MessageClient and fetch messages from the API"""
        super().__init__()
        self.messages = api.get('messages')

        self.types = {
            'info': '[green]Info[/]',
            'warn': '[yellow]Warning[/]',
            'maintenance': '[blue]Maintenance[/]'
        }

        logger.debug('Fetched messages from API')

    def show_messages(self) -> None:
        """Display unread messages"""

        if self.messages is not None:
            if settings.use_option('hide_messages'):
                read_message_ids = [int(id) for id in settings.get('read_messages', 'Loader').split(',')[:-1]]

                for message in self.messages.json():
                    if message['id'] not in read_message_ids and not message['hidden']:
                        if SAVE_MESSAGES:
                            settings.set('read_messages', settings.get('read_messages', 'Loader') + f'{message["id"]},', 'Loader')

                        local_tz = datetime.datetime.now(datetime.timezone.utc).astimezone().tzinfo
                        post_time = datetime.datetime.fromisoformat(message['post_at']).astimezone(local_tz)
                        time_difference = datetime.datetime.now(local_tz) - post_time

                        time_ago = self.calculate_time_ago(time_difference)

                        try:
                            print(f"\n{self.types[message['type']]} message from {post_time.strftime('%Y-%m-%d %H:%M:%S')} ({time_ago})\n{message['body']}\n")
                        except KeyError:
                            print(f"\n[gray]Unknown[/] type of message from {post_time.strftime('%Y-%m-%d %H:%M:%S')} ({time_ago})\n{message['body']}\n")

                self.shown = True
        else:
            logger.error('MessageClient error')

    @staticmethod
    def calculate_time_ago(time_difference: datetime.timedelta) -> str:
        """Calculate a human-readable time difference"""
        if time_difference < datetime.timedelta(minutes=1):
            return "just now"
        total_seconds = time_difference.total_seconds()
        if total_seconds < 3600:
            return f"{int(total_seconds // 60)} minutes ago"
        if total_seconds < 86400:
            return f"{int(total_seconds // 3600)} hours ago"
        return f"{time_difference.days} days ago"


messageclient = MessageClient()
