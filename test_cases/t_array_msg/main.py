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

    # ***
    inner = py_msg.Inner(3)
    inner2 = py_msg.Inner(1)

    # ***
    msg = py_msg.MsgFixedBaseArray.default()
    error_counter += write_or_test("val_array_default.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgFixedBaseArray(
        py_msg.SensorSource.TemperaturSensor, [1, 2, 3], [-2, 2, 0],[True, False, True],
        [-1, 123, 10], [1.1,2.2,123.456], [1.1, -1.1, 1.2],
        [py_msg.SensorSource.MovementSensor, py_msg.SensorSource.MovementSensor, py_msg.SensorSource.TemperaturSensor],
        [inner, inner2, inner])
    error_counter += write_or_test("val_array_val1.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgDynBaseArray.default()
    error_counter += write_or_test("val_dynarray_default.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgDynBaseArray(
        py_msg.ExampleEnum.E5, [1,2,3], [-2, 2, 0], [True, False, True],
        [-1, 123, 10], [1.1, 2.2, 123.456], [1.1, -1.1, 1.2],
        [py_msg.SensorSource.MovementSensor, py_msg.SensorSource.MovementSensor, py_msg.SensorSource.TemperaturSensor],
        [inner, inner, inner2])
    t = msg.val1
    t.append(4)
    msg.val1 = t
    error_counter += write_or_test("val_dynarray_val1.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgDynBaseArray.default()
    msg.val1 = [i&7 for i in range(13)]
    msg.val2 = [i&7 for i in range(23)]
    error_counter += write_or_test("val_dynarray_val2.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgLargeFixedArray.default()
    error_counter += write_or_test("val_large_array_default.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgLargeFixedArray.default()
    msg.val1 = [(i+1)&7 for i in range(len(msg.val1))]
    msg.val2 = [(1-(i&1)*2)*((i&7)>>1) for i in range(len(msg.val1))]
    msg.val3 = [(i&2)==2 for i in range(len(msg.val1))]
    error_counter += write_or_test("val_large_array_val1.py.dat", msg, data_id_replacement)


    print(f"\n* Total errors: {error_counter}")
    exit(error_counter)

if __name__ == "__main__":
    main()
