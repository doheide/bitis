from __future__ import annotations
from enum import Enum



# Enums

class SensorSource(Enum):
  TemperaturSensor = 1
  MovementSensor = 2
class ExampleEnum(Enum):
  E1 = 1
  E2 = 2
  E3 = 3
  E4 = 4
  E5 = 5
  E6 = 6
  E7 = 7
  E8 = 8
  E9 = 9



# *** Enums for oneof


# *** Messages

class Inner:
  def __init__(self, val2: int,) -> None: ...

  @staticmethod
  def default() -> Inner: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> Inner:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def val2(self) -> int: ...
  @val2.setter
  def val2(self, v: int) -> None: ...



class MsgFixedBaseArray:
  def __init__(self, param_1: SensorSource, val1: list[int], val2: list[int], val3: list[bool], val4: list[int], val5: list[float], val6: list[float], val7: list[SensorSource], val8: list[Inner],) -> None: ...

  @staticmethod
  def default() -> MsgFixedBaseArray: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> MsgFixedBaseArray:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def param_1(self) -> SensorSource: ...
  @param_1.setter
  def param_1(self, v: SensorSource) -> None: ...
  @property
  def val1(self) -> list[int]: ...
  @val1.setter
  def val1(self, v: list[int]) -> None: ...
  @property
  def val2(self) -> list[int]: ...
  @val2.setter
  def val2(self, v: list[int]) -> None: ...
  @property
  def val3(self) -> list[bool]: ...
  @val3.setter
  def val3(self, v: list[bool]) -> None: ...
  @property
  def val4(self) -> list[int]: ...
  @val4.setter
  def val4(self, v: list[int]) -> None: ...
  @property
  def val5(self) -> list[float]: ...
  @val5.setter
  def val5(self, v: list[float]) -> None: ...
  @property
  def val6(self) -> list[float]: ...
  @val6.setter
  def val6(self, v: list[float]) -> None: ...
  @property
  def val7(self) -> list[SensorSource]: ...
  @val7.setter
  def val7(self, v: list[SensorSource]) -> None: ...
  @property
  def val8(self) -> list[Inner]: ...
  @val8.setter
  def val8(self, v: list[Inner]) -> None: ...



