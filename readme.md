This is a wip ascii diagram editor.
I write a lot of these by hand, and while I generally enjoy it, I thought it'd
be a nice exercise in rust and some topology stuff.

What it currently does:

You can pipe this readme through `cargo run` and it will attempt to colour boxes blue and any lines yellow.

```
  ,------.
  |  hi  |
  '------'

Hello world

 ,---.,-----------.
 |   |',-.   nice |
 |   | | |  ,-----'
 '---' | |  |
       | |<-'
       '-'

 ,---------.    ,----->
 |   ,----.|----'
 |---|  0 ||-------->
 |   '----'|------.
 '---------'      |
                  v
```

What I'd like it to do:

editing, so be able to more easily make some of the following edits:
```
 ,---. ,---.  translate             ,---.
 |   | |   |  -------->  ,---.      |   |
 '---' '---'             |   |      '---'
                         '---'
 
 ,---.      ,---.  insert  ,---. ,--. ,---.
 |   |      |   |  ----->  |   | |  | |   |
 '---'      '---'          '---' '--' '---'

 ,---. ,---.  (local)  ,---. ,---.
 |   | |   |   scale   |   | |   |
 '---' |   |   ---->   |   | |   |
       '---'           '---' '---'
```
