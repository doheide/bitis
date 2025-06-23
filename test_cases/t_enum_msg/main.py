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

    msg = py_msg.MsgEnumOne.default()
    error_counter += write_or_test("val_enum_one_default.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgEnumOne(3, py_msg.SensorSource.MovementSensor)
    error_counter += write_or_test("val_enum_one_val1.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgEnumTwo.default()
    error_counter += write_or_test("val_enum_two_default.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgEnumTwo(33, py_msg.ExampleEnum.E8)
    error_counter += write_or_test("val_enum_two_val1.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgEnumOpt.default()
    error_counter += write_or_test("val_enum_opt_default.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgEnumOpt(3, py_msg.SensorSource.TemperaturSensor, py_msg.ExampleEnum.E8)
    error_counter += write_or_test("val_enum_opt_val1.py.dat", msg, data_id_replacement)

    print(f"\n* Total errors: {error_counter}")
    exit(error_counter)

if __name__ == "__main__":
    main()
