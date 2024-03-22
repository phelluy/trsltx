grammar latex;

latex: stuff;

stuff: (atom | construct)* ;

atom: command | comment | text ;  /* sans majuscules */

construct:  dmath | tmath | group | env;


dmath: '$$' stuff '$$' | '\\[' stuff '\\]' ;

tmath: '$' stuff '$' | '\\(' stuff '\\)' ;

group: '{' stuff '}' ;

env: Begin stuff End ;

comment: Comment ;
Comment: '%'~[\n]*'\n' ;
command: Command;
Command: '\\'[\\a-zA-Z]+ ;
text: Text ;
Text: (~[\\{}$%])+ ;
Begin: '\\begin{'[a-zA-Z]+'*'?'}' ;
End: '\\end{'[a-zA-Z]+'*'?'}' ;