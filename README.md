
# Gossyp

## Hello world in gossyp

```print "Hello, world"```

## In depth

Gossyp is a JSON-based language built to try to take advantage of the idea that
smaller programs are easier to maintain than bigger ones, by providing a means
to build large programs by combining small ones.

It has two core concepts. First is the 'Tool', which is a simple program that 
takes in JSON and outputs JSON. Second is the 'Environment' that makes it possible
to give tools names and look them up. Finally, there is a fairly simple 
'orchestration' language that can be used to tie tools together and build large
software from smaller pieces.

Tools can be seens as a slight modernisation of the idea of the command-line
program that most developers will be familiar with. JSON is a fairly nice language
to use in this context: computers already understand it so there's less need for
the constant parsing and reformatting that is performed when composing command
line utilities, and it is reasonably human-friendly so it can be written and
read directly.

The trick that makes the system work for compositing is that as well as JSON, 
tools also receive an environment so they can look up and call other tools. This
makes it relatively easy to write, say, an array sorting tool that can call out
to another tool to decide on element ordering.

This design has a lot of parallels with the 'RESTful' concept found in web design,
although gossyp eliminates the need to involve web servers so it is more convenient
when writing smaller programs and much faster to get set up. This form of programming
tends to eliminate complicated set-up procedures, which means that tests can be
specified purely in terms of input and output, which is a highly desirable situation
from a maintenance point of view, and also makes it easy to specify the behaviour
of a new tool in advance.

The use of JSON to communicate between tools means that they are weakly coupled.
This makes it easy to hide where a tool is running (in-process, in an external
process or over the internet) and also means that it's very easy to substitute
one tool for another, which can be a useful thing to do when developing a system
or upgrading an existing one.

Tools have no state of their own but may be used to modify state that lives 
elsewhere. The most obvious example of this is the `define` tool, which is the
standard way of creating new tools in an environment, but this also allows for
things like database access.
