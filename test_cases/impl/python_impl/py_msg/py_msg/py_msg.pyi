from __future__ import annotations
from enum import Enum



# Enums



# *** Enums for oneof




# *** Messages

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



class MsgSimpleBaseThreeInt:
  def __init__(self, param_1: int, param_2: int, param_3: int, param_4: int,) -> None: ...

  @staticmethod
  def default() -> MsgSimpleBaseThreeInt: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> MsgSimpleBaseThreeInt:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def param_1(self) -> int: ...
  @param_1.setter
  def param_1(self, v: int) -> None: ...
  @property
  def param_2(self) -> int: ...
  @param_2.setter
  def param_2(self, v: int) -> None: ...
  @property
  def param_3(self) -> int: ...
  @param_3.setter
  def param_3(self, v: int) -> None: ...
  @property
  def param_4(self) -> int: ...
  @param_4.setter
  def param_4(self, v: int) -> None: ...



class MsgSimpleTestBase:
  def __init__(self, param_1: int, param_2: bool, param_3: int, name: String,) -> None: ...

  @staticmethod
  def default() -> MsgSimpleTestBase: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> MsgSimpleTestBase:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def param_1(self) -> int: ...
  @param_1.setter
  def param_1(self, v: int) -> None: ...
  @property
  def param_2(self) -> bool: ...
  @param_2.setter
  def param_2(self, v: bool) -> None: ...
  @property
  def param_3(self) -> int: ...
  @param_3.setter
  def param_3(self, v: int) -> None: ...
  @property
  def name(self) -> String: ...
  @name.setter
  def name(self, v: String) -> None: ...



class MsgSimpleTestFP:
  def __init__(self, param_1: bool, fp: float, fpl: float,) -> None: ...

  @staticmethod
  def default() -> MsgSimpleTestFP: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> MsgSimpleTestFP:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def param_1(self) -> bool: ...
  @param_1.setter
  def param_1(self, v: bool) -> None: ...
  @property
  def fp(self) -> float: ...
  @fp.setter
  def fp(self, v: float) -> None: ...
  @property
  def fpl(self) -> float: ...
  @fpl.setter
  def fpl(self, v: float) -> None: ...



class MsgSimpleOpt:
  def __init__(self, param_1: int, param_2: bool, param_3: int, param_4: float,) -> None: ...

  @staticmethod
  def default() -> MsgSimpleOpt: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> MsgSimpleOpt:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def param_1(self) -> int: ...
  @param_1.setter
  def param_1(self, v: int) -> None: ...
  @property
  def param_2(self) -> bool: ...
  @param_2.setter
  def param_2(self, v: bool) -> None: ...
  @property
  def param_3(self) -> int: ...
  @param_3.setter
  def param_3(self, v: int) -> None: ...
  @property
  def param_4(self) -> float: ...
  @param_4.setter
  def param_4(self, v: float) -> None: ...



