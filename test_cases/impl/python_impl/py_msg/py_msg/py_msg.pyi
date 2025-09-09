from __future__ import annotations
from enum import Enum



# Enums



# *** Enums for oneof



class OO_MsgOoSimpleBase_Value:
  @staticmethod
  def new_int(int: int) -> OO_MsgOoSimpleBase_Value: ...
  @staticmethod
  def new_number(number: float) -> OO_MsgOoSimpleBase_Value: ...
  @staticmethod
  def new_true_false(true_false: bool) -> OO_MsgOoSimpleBase_Value: ...


class OO_MsgOoNestedBase_Value:
  @staticmethod
  def new_inner(inner: MsgSimpleBaseOneInt) -> OO_MsgOoNestedBase_Value: ...
  @staticmethod
  def new_number(number: float) -> OO_MsgOoNestedBase_Value: ...
  @staticmethod
  def new_true_false(true_false: bool) -> OO_MsgOoNestedBase_Value: ...



# *** Messages

class MsgOOSimpleBase:
  def __init__(self, id: int, value: OO_MsgOoSimpleBase_Value,) -> None: ...

  @staticmethod
  def default() -> MsgOOSimpleBase: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> MsgOOSimpleBase:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def id(self) -> int: ...
  @id.setter
  def id(self, v: int) -> None: ...
  @property
  def value_oo(self) -> OO_MsgOoSimpleBase_ValueEnum: ...
  @property
  def value_int(self) -> None | int: ...
  @value_int.setter
  def value_int(self, val: int) -> None: ...
  @property
  def value_number(self) -> None | float: ...
  @value_number.setter
  def value_number(self, val: float) -> None: ...
  @property
  def value_true_false(self) -> None | bool: ...
  @value_true_false.setter
  def value_true_false(self, val: bool) -> None: ...



class MsgSimpleBaseOneInt:
  def __init__(self, param_1: int,) -> None: ...

  @staticmethod
  def default() -> MsgSimpleBaseOneInt: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> MsgSimpleBaseOneInt:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def param_1(self) -> int: ...
  @param_1.setter
  def param_1(self, v: int) -> None: ...



class MsgOONestedBase:
  def __init__(self, id: int, value: OO_MsgOoNestedBase_Value,) -> None: ...

  @staticmethod
  def default() -> MsgOONestedBase: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> MsgOONestedBase:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def id(self) -> int: ...
  @id.setter
  def id(self, v: int) -> None: ...
  @property
  def value_oo(self) -> OO_MsgOoNestedBase_ValueEnum: ...
  @property
  def value_inner(self) -> None | MsgSimpleBaseOneInt: ...
  @value_inner.setter
  def value_inner(self, val: MsgSimpleBaseOneInt) -> None: ...
  @property
  def value_number(self) -> None | float: ...
  @value_number.setter
  def value_number(self, val: float) -> None: ...
  @property
  def value_true_false(self) -> None | bool: ...
  @value_true_false.setter
  def value_true_false(self, val: bool) -> None: ...



