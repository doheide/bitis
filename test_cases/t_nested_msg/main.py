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
    msg = py_msg.MsgWithInner.default()
    msg.imsg.param_1 = py_msg.SensorSource.MovementSensor
    error_counter += write_or_test("val_nested_default.py.dat", msg, data_id_replacement)

    # ***
    msgi = py_msg.MsgEnumOpt(1, py_msg.SensorSource.TemperaturSensor, py_msg.ExampleEnum.E3)
    msg = py_msg.MsgWithInner(2, msgi)
    error_counter += write_or_test("val_nested_val1.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgWithInner.default()
    msg.val = 3
    msg.imsg.param_1 = py_msg.SensorSource.TemperaturSensor
    msg.imsg.param_2 = py_msg.ExampleEnum.E7
    error_counter += write_or_test("val_nested_val2.py.dat", msg, data_id_replacement)


    # ***
    inner = py_msg.MsgEnumOpt(1, py_msg.SensorSource.TemperaturSensor, py_msg.ExampleEnum.E3)
    msgi = py_msg.MsgWithInner(2, inner)
    msg = py_msg.MsgWithTwoInner(47, msgi, None)
    error_counter += write_or_test("val_nested_two_val1.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgWithTwoInner.default()
    msg.imsg.imsg.val = 1
    error_counter += write_or_test("val_nested_two_val2py.dat", msg, data_id_replacement)


    print(f"\n* Total errors: {error_counter}")
    exit(error_counter)

if __name__ == "__main__":
    main()
