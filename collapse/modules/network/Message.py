import datetime

from rich import print

from ...developer import SAVE_MESSAGES
from ..storage.Settings import settings
from ..utils.Language import lang
from ..utils.Module import Module
from .API import api


class MessageClient(Module):
    """Client for retrieving and displaying messages"""

    def __init__(self) -> None:
        """Initialize the MessageClient and fetch messages from the API"""
        super().__init__()
        self.shown = False
        self.messages = api.get('messages')
        self.types = {
            'info': f'[green]{lang.t("messages.types.info")}[/]',
            'warn': f'[yellow]{lang.t("messages.types.warning")}[/]',
            'maintenance': f'[blue]{lang.t("messages.types.maintenance")}[/]'
        }
        
        self.debug(lang.t('messages.fetched'))

    def show_messages(self) -> None:
        """Display unread messages"""
        if self.messages is not None:
            if settings.use_option('hide_messages'):
                read_message_ids = set(int(id) for id in settings.get('read_messages', 'Loader').split(',')[:-1])
                local_tz = datetime.datetime.now(datetime.timezone.utc).astimezone().tzinfo
                current_time = datetime.datetime.now(local_tz)

                for message in self.messages.json():
                    if message['id'] not in read_message_ids:
                        if SAVE_MESSAGES:
                            read_message_ids.add(message['id'])
                            settings.set('read_messages', ','.join(map(str, read_message_ids)) + ',', 'Loader')

                        post_time = datetime.datetime.fromisoformat(message['post_at']).astimezone(local_tz)
                        time_difference = current_time - post_time
                        time_ago = self.calculate_time_ago(time_difference)

                        message_type = self.types.get(message['type'], '[gray]Unknown[/]')
                        print(f"\n{message_type} {lang.t('messages.message')} {post_time.strftime('%Y-%m-%d %H:%M:%S')} ({time_ago})\n{message['body']}\n")

                self.shown = True
        else:
            self.error(lang.t('messages.fetch-error'))

    @staticmethod
    def calculate_time_ago(time_difference: datetime.timedelta) -> str:
        """Calculate a human-readable time difference"""
        if time_difference < datetime.timedelta(minutes=1):
            return lang.t('messages.time-now')
        total_seconds = time_difference.total_seconds()
        if total_seconds < 3600:
            return lang.t('messages.time-minutes').format(int(total_seconds // 60))
        if total_seconds < 86400:
            return lang.t('messages.time-hours').format(int(total_seconds // 3600))
        return lang.t('messages.time-days').format(time_difference.days)

messageclient = MessageClient()