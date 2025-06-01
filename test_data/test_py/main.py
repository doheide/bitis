import bitis_msgs

def main():
    inner = bitis_msgs.Inner(2, bitis_msgs.Numbers.Three)
    print(inner)
    inner.val = 1
    oo = bitis_msgs.OO_ParamTestWithInner_Action.new_inner(inner)
    print(oo)
    msg = bitis_msgs.ParamTestWithInner(11, True, oo, None)
    print(msg)
    msg.param_2 = False
    print(msg)
    msg.param_1 = 1
    a = msg.param_2
    print(f"a={a}")
    print(msg)
    print(inner.val)


    # msg.inner.val = 3
    # msg.inner.num = bitis_msgs.Numbers.One
    # print(msg)

    b = msg.serialize()
    print(b)

    # ****
    r = bitis_msgs.ParamTestWithInner.deserialize(b"F\x06")
    print(r)
    r = bitis_msgs.ParamTestWithInner.deserialize(b"F\x11")
    print(r)


if __name__ == "__main__":
    main()
