#!/usr/bin/env python3

# run without argument, or read USAGE below

#
# CONFIG
#

VERBATIM_ENVS = ["verbatim", "Verbatim", "semiverbatim"]
CAPTURE_VERBATIMS = True

#
# LEXING
#

import re

class Location:

    def __init__ (self, o, l, c):
        self.offset = o # utf-8 offset
        self.line = l   # 0-based line number
        self.column = c # 0-based column number

    def after (self, s):
        o, l, c = self.offset, self.line, self.column
        for u in s:
            o += 1
            if u == "\n":
                l += 1
                c = 0
            else:
                c += 1
        return Location (o, l, c)

class SyntaxError (Exception):
    pass

class Lexer:

    def __init__ (self, text, begn):
        self.re = \
            re.compile (
                # constructs
                r'(?P<ENVBEGIN>\\begin\{(?P<BNAME>[A-Za-z]+\*?)\})'
                r'|(?P<ENVEND>\\end\{(?P<ENAME>[A-Za-z]+\*?)\})'
                r'|(?P<DMATHBEGIN>\\\[)'
                r'|(?P<DMATHEND>\\\])'
                r'|(?P<TMATHBEGIN>\\\()'
                r'|(?P<TMATHEND>\\\))'
                r'|(?P<GROUPBEGIN>\{)'
                r'|(?P<GROUPEND>\})'
                r'|(?P<DOLDOL>\$\$)'
                r'|(?P<DOL>\$)'
                # atoms
                r'|(?P<CNAME>\\([A-Za-z]+|.))'
                r'|(?P<COMMENT>%.*\n)'
                r'|(?P<TEXT>[^\\{}$%]+)'
            )
        self.text = text # full text as a string
        self.begn = begn # location where analysis started
        self.locn = begn # next location for scanning
        self.kind = None # from the regexp group
        self.tokn = None # text of the token (only name for envs)
        self.advance ()
    def error (self, msg):
        raise SyntaxError ("error: %s (at %d:%d-%d, looking at %s/%s)"
                           % (msg, self.begn.offset,
                              self.begn.line, self.begn.column,
                              self.kind, repr(self.tokn)))
    def __nexttoken (self):
        if self.kind == "$":
            pass
        elif self.locn.offset == len (self.text):
            self.tokn = self.kind = "$"
        elif (CAPTURE_VERBATIMS and
              self.kind == "ENVBEGIN" and self.tokn in VERBATIM_ENVS):
            d = self.text.find ("\\end{%s}" % self.tokn, self.locn.offset)
            if d == -1:
                self.error ("unclosed '%s' environment @ %d"
                            % (self.tokn, self.locn.offset))
            self.kind = "VERB"
            self.tokn = self.text[self.locn.offset:d]
            self.begn = self.locn
            self.locn = self.locn.after (self.tokn)
        else:
            m = self.re.match (self.text, self.locn.offset)
            if m is None:
                self.error ("lexer jammed @ %d" % (self.locn.offset))
            else:
                self.kind = m.lastgroup
                if self.kind in ["ENVBEGIN"]:
                    self.tokn = m.group ("BNAME")
                elif self.kind in ["ENVEND"]:
                    self.tokn = m.group ("ENAME")
                else:
                    self.tokn = m.group (0)
                self.begn = self.locn
                self.locn = self.locn.after (m.group (0))
    def advance (self):
        self.__nexttoken () # nothing is ignored here
    def closes (self, k, t):
        return (k == "ENVBEGIN" and self.kind == "ENVEND" and t == self.tokn or
                k == "DMATHBEGIN" and self.kind == "DMATHEND" or
                k == "TMATHBEGIN" and self.kind == "TMATHEND" or
                k == "GROUPBEGIN" and self.kind == "GROUPEND" or
                k == "DOLDOL" and self.kind == "DOLDOL" or
                k == "DOL" and self.kind == "DOL")

def test_lexing (src):
    lexer = Lexer (src, Location (0, 0, 0))
    while lexer.kind != '$':
        txt = lexer.tokn if len (lexer.tokn) < 16 else (lexer.tokn[:16]+"...")
        print (lexer.begn.offset, lexer.begn.line, lexer.begn.column,
               lexer.kind, ":", repr (txt))
        lexer.advance ()

#
# PARSING
#

KIND_DISPLAY = {
    # node type
    "ENVBEGIN": "ENV",
    "DMATHBEGIN": "DMATH", "DOLDOL": "DMATH",
    "TMATHBEGIN": "TMATH", "DOL": "TMATH",
    "GROUPBEGIN": "GROUP",
    "CNAME": "CNAME", "COMMENT": "COMMENT", "TEXT": "TEXT", "VERB": "VERB",
    # fake nodes
    "FILE": "FILE", "PREAMBLE": "PREAMBLE", "POSTAMBLE": "POSTAMBLE",
    "END": "END",
}

class AST:

    def __init__ (self, k, t, l):
        self.kind = k # actually, kind of opening token
        self.text = t # full text, or name of env
        self.locn = l # starting location
        self.subn = ( # list of sub-nodes
            None if k in ["CNAME", "COMMENT", "TEXT", "VERB"] else list ())

    def head (self):
        locs = "%05d:%04d-%02d" % (self.locn.offset,
                                   self.locn.line+1, self.locn.column+1)
        if len (self.text) > 16:
            txt = self.text[:8] + "[...]" + self.text[-8:]
        else:
            txt = self.text
        return locs, KIND_DISPLAY[self.kind], txt
    def dump (self, depth=0):
        locs, knd, txt = self.head ()
        print ("%s: %s%s: %s" % (locs, "    "*depth, knd, repr(txt)))
        if self.subn is not None:
            for s in self.subn:
                s.dump (depth+1)


def parse_nodes (lexer, stack):
    while not lexer.closes (stack[-1].kind, stack[-1].text):
        if lexer.kind in ["ENVBEGIN", "DMATHBEGIN", "TMATHBEGIN", "GROUPBEGIN",
                          "DOLDOL", "DOL"]:
            stack.append (AST (lexer.kind, lexer.tokn, lexer.begn))
            lexer.advance ()
            parse_nodes (lexer, stack)
        elif lexer.kind in ["CNAME", "COMMENT", "TEXT", "VERB"]:
            stack[-1].subn.append (AST (lexer.kind, lexer.tokn, lexer.begn))
            lexer.advance ()
        else:
            lexer.error ("wrong closing construct")
    stack[-1].subn.append (AST ("END", lexer.tokn, lexer.begn)) # trick
    lexer.advance ()
    n = stack.pop ()
    stack[-1].subn.append (n)
    
def parse_file (s):
    d = s.find ("\\begin{document}")
    if d == -1:
        raise SyntaxError ("error: \\begin{document} not found")
    stack = [AST ("FILE", "", None)]
    zero = Location (0, 0, 0)
    preamble = s[:d]
    stack[-1].subn.append (AST ("PREAMBLE", preamble, zero))
    lexer = Lexer (s, zero.after (preamble))
    stack.append (AST (lexer.kind, lexer.tokn, lexer.begn))
    lexer.advance ()

    parse_nodes (lexer, stack)

    postamble = s[lexer.begn.offset:]
    stack[-1].subn.append (AST ("POSTAMBLE", postamble, lexer.begn))
    stack[-1].locn = lexer.begn.after (postamble)
    return stack[-1]

#
# SELECTION
#

def sel_parse (s):
    # would be an ideal work for lambdas, but we can't use them here,
    # because Python is a fucking half-baked language wrt to closures
    # https://docs.python.org/3/reference/executionmodel.html#interaction-with-dynamic-features
    # instead we return a pair (kind, value) and defer some work until usage
    # (e.g., for comments -- see sel_apply)
    if s.startswith ("\\"):
        return ("CNAME", s)
    elif s.startswith ("m:"):
        return ("CNAME", "\\"+s[2:])
    elif s.startswith ("{") and s.endswith ("}"):
        return ("ENVBEGIN", s[1:-1])
    elif s.startswith ("e:"):
        return ("ENVBEGIN", s[2:])
    elif s.startswith ("%"):
        return ("COMMENT", s[1:])
    elif s.startswith ("c:"):
        return ("COMMENT", s[2:])
    else:
        raise SyntaxError ("error: invalid selector %s" % s)

def sel_apply (node, sel):
    if sel[0] == "COMMENT":
        return node.kind == "COMMENT" and node.text[1:].split()[0] == sel[1]
    else:
        return node.kind == sel[0] and node.text == sel[1]

def sel_select (node, lsels):
    for s in lsels:
        if sel_apply (node, s):
            return True
    return False

def select (lnodes, lsels):
    return [ i for i, n in enumerate (lnodes) if sel_select (n, lsels) ]

#
# USAGE
#

USAGE = r"""

Parses a LaTeX file, builds a syntax tree, selects top-level elements.
Usage:

    $PROG <input> tree
    $PROG <input> anchors selector [selector...]
    $PROG <input> chunks selector [selector...]

where <input> is either "-" for standard input, or any path. The LaTeX
file must be well-formed; the analysis is purely syntactic, no
interpretation is even attempted. Special syntaxes (delimited
arguments for \def, TikZ syntax, etc.) are completely unknwon and may
be considered raw text in favorable situations, or errors otherwise.
Your mileage may vary.

The syntax tree follows the following grammar:

    <file> := <pre> "\begin{document}" <stuff> "\end{document}" <post>
    <pre> := ... (node: PREAMBLE -- anything up to "\begin{document}")
    <post> := ... (node: POSTAMBLE -- anything after "\end{document}")
    <stuff> := (<atom> | <construct>)*
    <atom> := "\"<ID>        (node: CNAME)
           |  "%"<NOTNL>"\n" (node: COMMENT)
           |  <REGULAR>+     (node: TEXT)
    <construct> := "\begin{"ENVNAME"}" <stuff> "\end{"ENVNAME"}" (node: ENV)
                |  "\[" <stuff> "\]" (node: DMATH)
                |  "$$" <stuff> "$$" (node: same)
                |  "\(" <stuff> "\)" (node: TMATH)
                |  "$" <stuff> "$"   (node: same)
    <ID> := [a-zA-Z]+
    <NOTNL := .* (note: . excludes \n)
    <REGULAR> := [^\\{}$%]+ (note: anything that does not start a construct)
    <ENVNAME> := [a-zA-Z]+\*?

Constructs must be correctlty nested, and that is about all that is
verified. Note that \newenvironment will not work when wrapping
another environment. Put such definitions in the preamble, where they
are silently skipped over.


The first call form ("tree") prints the tree, with a node on each
line, preceded with location information in the form

    ooooo:llll-cc:

indicating utf-8 offset (ooooo), line-number (llll), and column-number
(cc). Line and column numbers start at one. The root element (FILE)
location is the length of the file (more precisely: the location of
the first character that would come after the file contents).


The second call form ("anchors") searches for top-level anchors (i.e.,
immediately inside document). It takes one or more selectors, which
are strings of the form:

    "\"<ID> or "m:"<ID>             (node: CNAME by name)
    "{"<ENVNAME>"}" or "e:"<ENVAME> (node: ENV by name)
    "%<STRING>" or "c:"<STRING>     (node: COMMENT containing that <STRING>)

(where <STRING> is anything not containing space). Both forms (e.g.,
"\section" and "m:section") are equivalent, but your shell may not
like unquoted versions of the first form.

Any top-level node matching any of the given selectors is... selected.
A selected node must start at the beginning of a line. The list of
selected node is printed with lines of the form:

    "line" llll "char" ooooo "kind" <KIND> "name" <TEXT>

where llll is the (one-based) line number, ooooo is the (zero-based)
utf-8 character (not byte) position in the file, <KIND> is the node
type ("ENV", "CNAME", etc.) and <TEXT> is some representation of the
node's appearance in the input (as a Python literal).


The third form ("chunks") does the same as the second, but adds
anchors right after "\begin{document}" and right before
"\end{document}", and lists intervals between anchors, each one on a
line of the form:

    "lines" llll eeee "chars" ooooo nnnnn

meaning lines between llll and eeee (included), or nnnnn chars
starting at ooooo (yes, nnnnn is a size). No other info is printed.


Final remark: verbatim environments cause trouble for parsing, which
comes as no surprise. The contents of known verbatim-like environments
is captured as a single chunk of text and kept in a node of type VERB.
The list of known verbatim-like environment names is kept in variable
VERBATIM_ENVS. You may have to update this variable (at the top of the
source file) if you use custom verbatim environments. The special
treatment of verbatims can be disabled alltogether, but then their
contents must follow the grammar above.

"""

#
# MAIN
#

import sys
import argparse

if len (sys.argv) == 1:
    print (USAGE.replace ("$PROG", sys.argv[0]))
    sys.exit (0)

# parse args
parser = argparse.ArgumentParser ()
parser.add_argument ("input")
subparsers = parser.add_subparsers (dest="cmd", required=True)

parser_lexer = subparsers.add_parser ("lexer")

parser_tree = subparsers.add_parser ("tree")

parser_anchors = subparsers.add_parser ("anchors")
parser_anchors.add_argument ("selector", nargs="+")

parser_chunks = subparsers.add_parser ("chunks")
parser_chunks.add_argument ("selector", nargs="+")

args = parser.parse_args ()
#print (repr (args))

# shamelessly slurp everything in
if args.input == "-":
    src = sys.stdin.read ()
else:
    with open (args.input) as f:
        src = f.read()

# utility
truncrepr = lambda s, l: repr (s) if len (s) <= l else repr (s[:l]) + "[...]"

# process
try:
    if args.cmd == "lexer":
        test_lexing (src)
    elif args.cmd == "tree":
        root = parse_file (src)
        root.dump ()
    elif args.cmd == "anchors":
        root = parse_file (src)
        body = root.subn[1].subn
        lsels = [ sel_parse (s) for s in args.selector ]
        anchors = select (body, lsels)
        for a in anchors:
            node = body[a]
            txt = truncrepr (node.text, 32)
            print ("line %d char %d kind %s name %s"
                   % (node.locn.line+1, node.locn.offset, node.kind, txt))
    elif args.cmd == "chunks":
        root = parse_file (src)
        body = root.subn[1].subn
        start = body[0]
        if start.kind != "TEXT" or start.text[0] != "\n":
            raise SyntaxError ("error: trailing garbage after \begin{document}")
        lsels = [ sel_parse (s) for s in args.selector ]
        anchors = select (body, lsels)
        for a in anchors+[-1]:
            loc = body[a].locn
            if loc.column != 0:
                raise SyntaxError ("error: anchor not in column 0 (%d:%d)"
                                   % (loc.line, loc.column))
        nodes = [AST (start.kind, start.text[1:], start.locn.after ("\n"))] \
            + [ body[a] for a in anchors ] + [body[-1]]
        n = 0
        while nodes[n].kind != "END":
            o = nodes[n].locn.offset
            l = nodes[n+1].locn.offset - o
            print ("lines %d %d chars %d %d" %
                   (nodes[n].locn.line+1, nodes[n+1].locn.line, o, l))
            n += 1
    r = 0
except SyntaxError as e:
    print (str (e))
    r = 1

sys.exit (r)

# TODO
# - option/config for adding verbatim envs: exec (open (<path>.chew).read ())
# - actions: round-trip?
