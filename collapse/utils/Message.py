from .Logger import logger
from .API import api
from .Settings import settings
from rich import print
from datetime import datetime, timedelta, timezone

class MessageClient:
    """Client for get messages"""
    def __init__(self):
        self.messages = api.get('messages').json()

        logger.debug('Get messages')

    def show_messages(self):
        """Show messages (if not readed)"""

        for message in self.messages:
            # If remote message not in local message

            if not message['id'] in [int(id) for id in settings.get('read_messages').split(',')[:-1]]:
                settings.set('read_messages', settings.get('read_messages') + f'{message['id']},')

                post_time = datetime.fromisoformat(message['post_at']).astimezone(timezone.utc)
                now = datetime.now(timezone.utc)
                time_difference = now - post_time

                if time_difference < timedelta(minutes=1):
                    time_ago = "just now"
                elif time_difference < timedelta(hours=1):
                    minutes_ago = time_difference.seconds // 60
                    time_ago = f"{minutes_ago} minutes ago"
                elif time_difference < timedelta(days=1):
                    hours_ago = time_difference.seconds // 3600
                    time_ago = f"{hours_ago} hours ago"
                else:
                    days_ago = time_difference.days
                    time_ago = f"{days_ago} days ago"

                print(f"\nNew message from developer {post_time.strftime('%Y-%m-%d %H:%M:%S')} ({time_ago})\n{message['body']}\n")

messageclient = MessageClient()