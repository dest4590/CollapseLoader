import datetime

from collapse.modules.network.Message import Messages


def test_calculate_time_ago():
    client = Messages()
    assert client.calculate_time_ago(datetime.timedelta(seconds=30)) == "just now"
    assert client.calculate_time_ago(datetime.timedelta(minutes=5)) == "5 minutes ago"
    assert client.calculate_time_ago(datetime.timedelta(hours=2)) == "2 hours ago"
    assert client.calculate_time_ago(datetime.timedelta(days=1)) == "1 days ago"