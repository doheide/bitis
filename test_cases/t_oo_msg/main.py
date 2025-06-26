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
    msg = py_msg.MsgOOSimpleBase.default()
    error_counter += write_or_test("val_oosimple_default.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgOOSimpleBase(53, py_msg.OO_MsgOoSimpleBase_Value.new_number(1.23))
    error_counter += write_or_test("val_oosimple_val1.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgOOSimpleBase.default()
    msg.id = 54
    msg.value_int = 3
    error_counter += write_or_test("val_oosimple_val2.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgOOSimpleBase(55, py_msg.OO_MsgOoSimpleBase_Value.new_true_false(True))
    error_counter += write_or_test("val_oosimple_val3.py.dat", msg, data_id_replacement)

    # ***
    msg = py_msg.MsgOONestedBase.default()
    error_counter += write_or_test("val_oonested_default.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgOONestedBase.default()
    msg.id = 2
    msg.value_inner = py_msg.MsgSimpleBaseOneInt(1111)
    error_counter += write_or_test("val_oonested_val1.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgOONestedBase(3, py_msg.OO_MsgOoNestedBase_Value.new_number(123.456))
    error_counter += write_or_test("val_oonested_val2.py.dat", msg, data_id_replacement)


    print(f"\n* Total errors: {error_counter}")
    exit(error_counter)

if __name__ == "__main__":
    main()

