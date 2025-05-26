

# Demonstration of a basic String Concatenation service

The code in this crate allows one to execute locally [a demo of Ledgera](https://docs.ledgera.tech/docs/versions/v_0_1/4_at_work/), which runs a very simple string concatenation service.


## Principle

This demo consists in:
- deploying a ledgera network with 4 nodes
- launching 4 terminals through which one can observe the behavior of each node
- 2 of those terminals also serve as clients through which users may interact with the ledgera network via a Text User Interface (TUI)
- a minimalist toy application-layer runs over this Ledgera network, it allows one to concatenate strings

## Prerequisites 

Refer to [the Ledgera README](../../../README.md).

## Running the script

Open a terminal in the same folder as this present README and simply type (depending on your OS):

<table>
<tr>
<td> UNIX </td> <td> WINDOWS </td>
</tr>
<tr>
<td> 

``` 
python launch_test.py
```
</td>
<td>

``` 
python.exe .\launch_test.py
```
</td>
</table>

## Text User Interface for the clients

Two of the four terminals, associated with "node1" and "node3" provide a Text User Interface (TUI).
This TUI allows one to enter commands so that the corresponding node send requests to the ledgera network.

In the sections below, we describe 4 videos that illustrate the use of the TUI.
A more detailed description of the TUI can be found in [a dedicated README](../test_basic_tui/README.md).


### Storage request

Here, we have a user submit a storage request on the **Client** component running on "node1" and then we have another user
retrieve the corresponding Proof Of Storage and value from the **Client** component running on "node3":

https://www.youtube.com/watch?v=Apmmodq0Llw


### Simple computation on two raw concrete values

Here, we have a user submit a computation request on the **Client** component running on "node1".
This instance of a computation corresponds to concatenating two raw string values that are exchanged in clear.

https://www.youtube.com/watch?v=lo3gfIcUcgs

### Simple computation with an argument being a reference to the storage

Here, we have a user submit a storage request on the **Client** component running on "node3".
Then, another user submits a computation request on the **Client** component running on "node1".
This time, instead of having two raw values as arguments, the computation instance is such that one of the arguments
is instead a reference to the value previously stored in the storage.

https://www.youtube.com/watch?v=BsEJUGUn-bs


### Computation with an argument that is not known in advance

Below, we have a user submit a computation request on the **Client** component running on "node1".
This time, when specifying the computation, we de not provide all the arguments.
Instead, one of the arguments is an unknown which concrete value will be eventually agreed-upon by the system.

As a result, as long as the agreement has not yet been reached, the computation cannot be be executed.

Later in the video, we have another user propose a value for the missing argument via submitting an argument proposal on the **Client** component running on "node3".

Upon that value being agreed upon, the computation can be executed and the proof of its integrity eventually produced.

https://www.youtube.com/watch?v=clJz3fWe8mg






# Configure your own network

This crate contains 2 binaries in "./src/bin":
- a "testnet_maker" binary with which one can initialize PKI material to run several communicating ledgera nodes
- a "configurable_node_runner" binary with which one can configure and launch a ledgera node

## Details on the pre-configured demo

The [pre-configured demo](#simple-pre-configured-demo) uses the "testnet_maker" and "configurable_node_runner" binaries to deploy and run a specific network of 4 nodes such that:
- node 1 is a "client" and "voter"
- node 2 is a "storage" and "voter"
- node 3 is a "client" and "storage" and "voter"
- node 4 is a "voter" and "logger"

The video below illustrates how to manually start the pre-configure demo (instead of using the Python script):

https://www.youtube.com/watch?v=NKY4a28VHXg&t=57s


## How to configure your own network

### Building the executables

To build the 2 executables type:
```
cargo build --release
```

### PKI initialization

To configure a testnet with X nodes type in:
``` 
../../target/release/testnet_maker X
```

This will generate in the "./testnet" folder X distinct folders 
named "node1", "node2", ..., "nodeX" in each of which we have :
- a unique "private_key.txt" file that contains the private key of the corresponding node
- a "known_participants.txt" file which contain the public keys of all X nodes 

### Configuring and launching a node

After having generated the "testnet" folder, launch the corresponding nodes using the following command (where Y is in 1..X, and ROLES indicates the components one want to run on the corresponding node):
``` 
../../target/release/configurable_node_runner nodeY ROLES ./testnet/nodeY
```

ROLES is a string that may contain any of the following characters:
- 'v' if one wants the node to act as a voter
- 's' if one wants the node to act as a storage
- 'c' if one wants the node to act as a client
- 'l' if one wants the node to act as a logger

Please keep in mind that for the network to function a sufficient proportion of nodes have to play certain roles (see TODO link other README) for details.



# Test Use Cases

This crate contains minimal [application use cases](https://docs.ledgera.tech/docs/versions/v_0_1/1_concepts/#application-use-case) for integration tests 
implementations for Ledgera's computation framework.

In particular we define a computation by setting three parameters : the operation definition and the input's domain (input's type and the predicate)

## Trivial String Use Case

A simple test implementation featuring:

- 1 Data Type: `StrConcatData` - wraps a string
- 1 Operation: `Concat` - concatenates two strings
- 1 Predicate: `StringLongerThan(n)` - checks whether a string has a length greater than n.

Enter `o` in TUI to show available operations/predicates for compute in the current use case