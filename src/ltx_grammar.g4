grammar ltx_grammar;

latex: '\\begin{document}' stuff '\\end{document}';

stuff: (atom | construct)* ;

atom: command | comment | text ;

construct:  dmath | tmath | group ;

command: '\\' name ;

dmath: '$$' stuff '$$' | '\\[' stuff '\\]' ;

tmath: '$' stuff '$' | '\\(' stuff '\\)' ;

group: '{' stuff '}' ;

comment: '%' '()*?' '\n' ;

name: '[a-zA-Z]+' ;

text: ~'\\' ~'{' ~'}' ~'$' ~'%' + ;
