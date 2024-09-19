import logging
import os
import signal

import click
from flask import Flask, request

from collapse.modules.utils.ClientManager import client_manager
from collapse.modules.utils.Module import Module
from collapse.modules.storage.Settings import settings

app = Flask('CollapseLoader Server')

class SdkServer(Module):
    """SDK server for manipulating loader using HTTP requests"""
    
    def __init__(self, disable_logging: bool = True):
        super().__init__()
        self.app = app

        if disable_logging:
            log = logging.getLogger('werkzeug')
            log.disabled = True
            
            click.echo = lambda *args, **kwargs: None
            click.secho = lambda *args, **kwargs: None

    def run(self, host='127.0.0.1', port=9090, debug=False):
        """Start the server"""
        self.info(f"Server started on http://{host}:{port}")
        self.app.run(host=host, port=port, debug=debug)

    @app.route('/run', methods=['POST'])
    def client_run():
        """Start a client by name"""
        name = request.json.get('name')
        
        if not name:
            return 'Missing "name" parameter', 400

        client = client_manager.get_client_by_name(name)
        
        if client:
            client.run()
            return 'Client started', 200
        else:
            return 'Client not found', 404

    @app.route('/settings', methods=['GET'])
    def get_settings():
        """Get all settings"""
        with open(settings.config_path, 'r') as file:
            return file.read(), 200

    @app.route('/settings', methods=['POST'])
    def update_settings():
        """Update settings by key, value, header"""
        settings.set(request.json.get('key'), request.json.get('value'), request.json.get('header'))
        return 'Settings updated', 200

    @app.route('/shutdown', methods=['POST'])
    def stop_server():
        os.kill(os.getpid(), signal.SIGINT)
        return 'Server shutting down...', 200

"""
Server endpoints:
* /run - Start a client by name
* /settings (GET) - Get all settings
* /settings (POST) - Update settings by key, value, header
* /shutdown - Shutdown the server
"""

server = SdkServer()