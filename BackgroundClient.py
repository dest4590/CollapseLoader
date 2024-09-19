import socket

HOST = 'localhost'
PORT = 9090

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as client_socket:
    client_socket.connect((HOST, PORT))
    print("Connected to server")
    
    message = input('Enter message: ')
    client_socket.sendall(message.encode('utf-8'))
    print(f"Sent: {message}")