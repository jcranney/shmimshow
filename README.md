# `shmimshow`

## Intro
`shmimshow` aims to provide real-time monitoring of `ImageStreamIO` format
shared-memory data, with as little installation headache as possible and all
batteries included. The project is in the VERY early stages, so if you need to
use it soon then maybe stick to [pyMilk](https://github.com/milk-org/pyMilk)
or [xaosim](https://github.com/fmartinache/xaosim).

My goal is to arrive at a one-line installation without needing to think about
dependencies, possibly via `pip`. Then to provide a command line callable
application, with something like:
```bash
shmimshow "<name>.im.shm"
```
This is already the case with the packages listed above, but installation of
those is a bit rough around the edges. If you're reading this, and you know
of something as simple as I'm searching for, please reach out.

## Deep background (feel free to skip)
Where to begin...

The prolific [`ImageStreamIO`](https://github.com/milk-org/imagestreamio) library
forms the backbone of a huge number of Adaptive Optics real-time control pipelines,
and thanks to it's open-sourcedness, the actual number of systems that depend
on ISIO will never be known. That library is written in C, and is very often
used alongside [`milk`](https://github.com/milk-org/milk), which provides another
layer of abstraction in C++, with easy to use and mostly pain-free Python wrappers.

As a personal journey, I have been learning rust, and appointed myself as an
unsolicited *rustifier* of `ImageStreamIO`, as a hobby project. I'm an instrument control
specialist by day, so this choice is partly an exercise in "stick to what you know"
and partly a demonstration of my poor work-life boundaries. I should note that at the
time of writing this, none of my work projects allow rust, favouring Paleolithic
technologies like C and Python. My superiors won't be offended by this sentiment,
as any of them who are reading a github README are already aware of my feelings
on the matter, and possibly rustaceans themselves. But I digress.

My *rustification* of `ImageStreamIO` forms the
[`risio`](https://github.com/jcranney/risio) crate. I strive to implement as much
of that crate in rust, and at the time of writing, I no longer have an upstream
dependency on `ImageStreamIO`. Eventually, I'd like to use rust at work to
implement more reliable, performant, and safe control systems, but this will
certainly depend on a reliable port of `ImageStreamIO` existing in rust.
I guess my logic here is that if I can build up `risio` in my own time,
for free, and make it available to my day-person, then I will eventually have
a better time on work projects. I'm fully aware that this is sort of 
"exploitation of myself by myself for my employer", but whatareyougoingtodo...

I figure that building tooling *around* `risio` will help the development of
`risio`-proper, so that's mostly what this project is. The fact that there's a 
need for such a tool (in my opinion) is secondary.

Feel free to make suggestions or contribute through the standard means.