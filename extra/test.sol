pragma solidity ^0.5.0;

contract Inner {
    event InnerEvent(uint256);
    
    function docall(uint256 foo)  external  {
        emit InnerEvent(foo);
    }    
}

contract Outer {
    
    event OuterEvent(string);
    Inner inner;
    
    constructor() public {
        emit OuterEvent("constructor");
    }
    function create() external {
        inner = new Inner();    
    }
    function docall() external {
        inner.docall(11);
        emit OuterEvent("called");
    }
}