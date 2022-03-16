import socket
import numpy as np
import struct
import time
from tqdm import tqdm

HOST, PORT = "127.0.0.1", 50000
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

data = np.arange(0, 10e6, dtype=np.uint64)
print(data)
size = 8

for i in tqdm(range(50_000)):
    a = data[(i * size):((i + 1) * size)]
    print(a)
    #sendCmd = struct.pack("%sB" % size, *a)
    sendCmd = struct.pack("%sQ" % size, *a)
    sock.sendto(sendCmd, (HOST, PORT))
    # progress = int(i / total * 100)
    # if progress != 0 and progress % 5 == 0 and progress != last_progress:
    #    print(f'Progress {i / total * 100:.1f}%')
    #    last_progress = progress
    time.sleep(1)