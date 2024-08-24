import datetime

from rich import print

from ...static import SAVE_MESSAGES
from ..storage.Settings import settings
from ..utils.Module import Module
from .API import api


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
        self.debug('Fetched messages from API')

    def show_messages(self) -> None:
        """Display unread messages"""
        if self.messages is not None:
            if settings.use_option('hide_messages'):
                read_message_ids = set(int(id) for id in settings.get('read_messages', 'Loader').split(',')[:-1])
                local_tz = datetime.datetime.now(datetime.timezone.utc).astimezone().tzinfo
                current_time = datetime.datetime.now(local_tz)

                for message in self.messages.json():
                    if message['id'] not in read_message_ids and not message['hidden']:
                        if SAVE_MESSAGES:
                            read_message_ids.add(message['id'])
                            settings.set('read_messages', ','.join(map(str, read_message_ids)) + ',', 'Loader')

                        post_time = datetime.datetime.fromisoformat(message['post_at']).astimezone(local_tz)
                        time_difference = current_time - post_time
                        time_ago = self.calculate_time_ago(time_difference)

                        message_type = self.types.get(message['type'], '[gray]Unknown[/]')
                        print(f"\n{message_type} message from {post_time.strftime('%Y-%m-%d %H:%M:%S')} ({time_ago})\n{message['body']}\n")

                self.shown = True
        else:
            self.error('MessageClient error')

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