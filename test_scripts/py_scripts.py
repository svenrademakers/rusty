from py_annotation import *

def function_name():
    print("simple py func")

def Sven_for_life(asdf):
    print("asdfasfdsd py func")

def watIsDeze():
    print("wat")

# example starts here
@flaunch(wat="Print Statement", number="Given Number")
def myfunc(wat: int, number: str):
    """document this func"""
    print(wat)

@flaunch(s="sdf")
def que():
 print("sdfasd")
