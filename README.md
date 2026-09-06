A small binary to run in the background that updates local git state in reaction
to the gerrit event stream.
Combined with a per change jj repo with a shared bare git repo, we continuously track the state of changes of gerrit.
