# ELF to LE converter.

this is awful code, but has worked for me.
for more info check my site: https://ceionia.com/le-exe

you can find an example project for this on my site: https://ceionia.com/git/lucia/rust-le-demo
or on github: https://github.com/LCeionia/rust-le-demo

usage is `elf2le INPUT_FILE`, outputs `a.exe` as LE output, intended for the DOS/32A extender, haven't tested on anything else. you can run the generated executable in DOS with `dos32a a.exe`, and can make a standalone executable with the `sc` utility provided by DOS/32A. file `new.elf` is used as an intermediate, so don't call something that in the working directory or it'll get overwritten.

it doesn't support things that both formats support.
the code is absolute awful spaghetti.
i intended on doing something better or even just improving it but never did.
may the code gods have mercy on me for releasing this.

Copyright (c) 2023 Lucia Ceionia

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
