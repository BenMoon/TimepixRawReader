import numpy as np
import struct
import timeit

sendCMD = struct.pack("%sQ" % 50000, *np.arange(50000, dtype=np.uint64))

@timeit
def convert_bytes_integers():
   data = np.frombuffer(sendCMD, dtype=np.uint64)

if __name__ == '__main__':
    n = 100
    print("The sum of the first {} numbers is: {}".format(n, convert_bytes_integers()))


# timeit.timeit("numpy.frombuffer(sendCMD, dtype=np.uint64)", globals=globals())