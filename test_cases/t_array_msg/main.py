import sys
import py_msg
from pathlib import Path

epath = Path(sys.argv[0]).parent
sys.path.append(str(epath))

from helper import *


def main():
    data_id_replacement = None
    if len(sys.argv) > 1:
        data_id_replacement = sys.argv[1]

    error_counter = 0

    msg = py_msg.MsgFixedBaseArray(py_msg.SensorSource.MovementSensor, [1, 2, 3], [-2, 2, 0],
                                   [True, False, True], [-1, 123, 10],
                                   [1.1,1.2,33.4], [-1.2, 2, 6],
                                   [py_msg.SensorSource.MovementSensor, py_msg.SensorSource.MovementSensor,
                                    py_msg.SensorSource.TemperaturSensor],
                                   [py_msg.Inner.default(), py_msg.Inner(3), py_msg.Inner.default()]
                                   )
    msg.val1 = [1,2,4]
    print(f"la: {msg.val1}")
    print(f"li: {msg.val1[2]}")
    print(f"lu: {msg.val7[2]}")
    print(msg.val8[1])
    t = msg.val8[0]
    t.val2 = 1
    msg.val8 = [t, msg.val8[1], msg.val8[2]]

    error_counter += write_or_test("val_simple_one_int.py.dat", msg, data_id_replacement)



    print(f"\n* Total errors: {error_counter}")
    exit(error_counter)

if __name__ == "__main__":
    main()
