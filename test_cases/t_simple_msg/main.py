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

    msg = py_msg.MsgSimpleBaseOneInt.default()
    msg.param_1 = 1122
    error_counter += write_or_test("val_simple_one_int.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgSimpleBaseThreeInt(1122, 3, 223, 33)
    msg.param_3 = 3
    error_counter += write_or_test("val_simple_three_int.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgSimpleTestBase.default()
    error_counter += write_or_test("val_simple_default.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgSimpleTestBase(999, True, -13, "lalalililolo")
    error_counter += write_or_test("val_simple_param_set1.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgSimpleTestFP(True, 0.1)
    error_counter += write_or_test("val_simple_test_fp.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgSimpleOpt.default()
    error_counter += write_or_test("val_simple_opt_default.py.dat", msg, data_id_replacement)

    msg = py_msg.MsgSimpleOpt(223, True, 1234, None)
    error_counter += write_or_test("val_simple_opt_valset1.py.dat", msg, data_id_replacement)

    print(f"\n* Total errors: {error_counter}")
    exit(error_counter)

if __name__ == "__main__":
    main()

