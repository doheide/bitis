from __future__ import annotations



# Enums

class Numbers(Enum):
  """
   Test comment for Enum

  """
  One = 1
  Two = 2
  Three = 3
  Four = 4



# *** Enums for oneof
class OO_ParamTestWithInner_ActionEnum(Enum):
  Inner = 1
  Val = 2

class OO_ParamTestWithInner_Action:
  @staticmethod
  def new_inner(inner: Inner) -> OO_ParamTestWithInner_Action: ...
  @staticmethod
  def new_val(val: int) -> OO_ParamTestWithInner_Action: ...

# *** Messages

class Inner:
  """
   Test comment for Inner

  """

  def __init__(self, val: int, num: Numbers,) -> None: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @classmethod
  def deserialize(cls, data: bytes) -> Inner:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def val(self) -> int: ...
  @val.setter
  def val(self, v: int) -> None: ...
  @property
  def num(self) -> Numbers: ...
  @num.setter
  def num(self, v: Numbers) -> None: ...



class ParamTestWithInner:
  def __init__(self, param_1: int, param_2: bool, action: OO_ParamTestWithInner_Action) -> None: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @classmethod
  def deserialize(cls, data: bytes) -> ParamTestWithInner:
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
  def action_oo(self) -> OO_ParamTestWithInner_ActionEnum: ...
  @property
  def action_inner(self) -> None | Inner: ...
  @action_inner.setter
  def action_inner(self, val: Inner) -> None: ...
  @property
  def action_val(self) -> None | int: ...
  @action_val.setter
  def action_val(self, val: int) -> None: ...



