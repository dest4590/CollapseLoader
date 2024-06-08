import datetime

from rich import print

from ..static import SAVE_MESSAGES
from .API import api
from .Logger import logger
from .Settings import settings


class MessageClient:
    """Client for get messages"""

    shown = False

    def __init__(self):
        self.messages = api.get('messages').json()
        self.types = {
            'info': '[green]Info[/]',
            'warn': '[yellow]Warning[/]',
            'maintenance': '[blue]Maintenance[/]'
        }

        logger.debug('Get messages')

    def show_messages(self):
        """Show messages (if not readed)"""

        for message in self.messages:
            # If remote message not in local message

            if not message['id'] in [int(id) for id in settings.get('read_messages').split(',')[:-1]]:
                if SAVE_MESSAGES:
                    settings.set('read_messages', settings.get('read_messages') + f'{message["id"]},')

                local_tz = datetime.datetime.now(datetime.timezone.utc).astimezone().tzinfo  # Get local timezone from system
                post_time = datetime.datetime.fromisoformat(message['post_at']).astimezone(local_tz)
                now = datetime.datetime.now(local_tz)
                time_difference = now - post_time

                if time_difference < datetime.timedelta(minutes=1):
                    time_ago = "just now"
                else:
                    total_seconds = time_difference.total_seconds()
                    if total_seconds < 3600:
                        time_ago = f"{int(total_seconds // 60)} minutes ago"
                    elif total_seconds < 86400:
                        time_ago = f"{int(total_seconds // 3600)} hours ago"
                    else:
                        time_ago = f"{time_difference.days} days ago"

                try:
                    print(f"\n{self.types[message['type']]} message from {post_time.strftime('%Y-%m-%d %H:%M:%S')} ({time_ago})\n{message['body']}\n")
                except KeyError:
                    print(f"\n[gray]Unknown[/] type of message from {post_time.strftime('%Y-%m-%d %H:%M:%S')} ({time_ago})\n{message['body']}\n")
                    
        self.shown = True
messageclient = MessageClient()