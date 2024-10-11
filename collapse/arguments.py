import argparse

parser = argparse.ArgumentParser(description='CollapseLoader arguments')
parser.add_argument('-v', action='store_true', help='Enable debug logging')
parser.add_argument('--level', help='Set log level', choices=['debug', 'info', 'warning', 'error', 'critical'])
parser.add_argument('--disable-analytics', action='store_true', help='Disable analytics')
parser.add_argument('--timeout', type=int, help='Set timeout for network requests')
parser.add_argument('--crash', action='store_true', help='Force crash')
parser.add_argument('--api-url', help='Set API URL')
parser.add_argument('--lang', help='Set language')

# LINK - collapse/modules/sdk/SdkServer.py
parser.add_argument('--server', action='store_true', help='Run the SDK server')
parser.add_argument('--port', type=int, help='Set port for SDK server')
parser.add_argument('--server-debug', action='store_true', help='Enable debug mode for SDK server')
parser.add_argument('--no-logs', action='store_true', help='Disable logs for SDK server')

args, unknown = parser.parse_known_args()
enabled_args = {k: v for k, v in args._get_kwargs() if v is not None and v is not False}