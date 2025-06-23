from pathlib import Path




def write_or_test(fn_name: str, msg, data_id_replacement: str | None):
    t_msg = type(msg)
    (data, total_bits, total_bytes)  = msg.serialize()
    print(f"* total_bits: {total_bits}, total_bytes: {total_bytes}")

    msgd = t_msg.deserialize(data)
    print(msgd)

    if data_id_replacement is None:
        print(f"writing {fn_name}")
        Path(fn_name).write_bytes(data)
    else:
        fn_name = fn_name.replace(".ps.", f".{data_id_replacement}.")
        print(f"reading {fn_name}")
        data = Path(fn_name).read_bytes()

        msgds = t_msg.deserialize(data)
        print(msgds)
        if repr(msgd) == repr(msgds):
            print("** ok")
            return 0
        print("** FAILED!")
        return 1
    return 0
