import datetime

from collapse.utils.Message import MessageClient


def test_calculate_time_ago():
    client = MessageClient()
    assert client.calculate_time_ago(datetime.timedelta(seconds=30)) == "just now"
    assert client.calculate_time_ago(datetime.timedelta(minutes=5)) == "5 minutes ago"
    assert client.calculate_time_ago(datetime.timedelta(hours=2)) == "2 hours ago"
    assert client.calculate_time_ago(datetime.timedelta(days=1)) == "1 days ago"
