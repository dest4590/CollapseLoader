import requests


class SdkServerClient:
    """Client for interacting with the sdk server"""

    def __init__(self, port):
        self.url = f"http://localhost:{port}/"

    def post(self, path: str, data: dict = {}) -> str:
        """Send a POST request to the server"""
        request = requests.post(self.url + path, json=data)
        return [request.text, request.status_code]

    def get(self, path):
        """Send a GET request to the server"""
        return requests.get(self.url + path).text

    def start_client(self, name: str):
        """Start a client by name"""
        return self.post("run", {"name": name})

    def get_settings(self):
        """Get all settings"""
        return self.get("settings")

    def get_setting(self, key: str, header: str = "Options"):
        """Get a single setting by key and header"""
        return self.get("setting", {"key": key, "header": header})

    def set_setting(self, key: str, value: str, header: str = "Options"):
        """Set a setting by key, value, header"""
        return self.post("settings", {"key": key, "value": value, "header": header})

    def stop_server(self):
        """Stop the server"""
        return self.post("shutdown")


client = SdkServerClient(9090)

"""
Example usage:

# Start a client by name
client.start_client('client1')

# Get all settings
client.get_settings()

# Get a single setting by key and header
client.get_setting('key1', 'Options')

# Set a setting by key, value, header
client.set_setting('key1', 'value1', 'Options')

# Stop the server
client.stop_server()
"""
