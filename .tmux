#!/bin/sh
set -e

# change this to the name of this project
TMUX_SESS_NAME=git-lab-rust
# change this to the name of the first tmux window
TMUX_WIN1_NAME=vim
TMUX_WIN2_NAME=shell
TMUX_WIN3_NAME=gitlab-lib
TMUX_WIN4_NAME=local-runners
TMUX_WIN5_NAME=rust-build-docker

export NVIM_LISTEN_ADDRESS=/tmp/nvimsocket.$TMUX_SESS_NAME

if tmux has-session -t=$TMUX_SESS_NAME 2> /dev/null; then
  tmux attach -t $TMUX_SESS_NAME
  exit
fi

tmux new-session -d -s $TMUX_SESS_NAME -n $TMUX_WIN1_NAME -x $(tput cols) -y $(tput lines)

# create vim window
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN1_NAME "vim -c CommandTBoot" Enter

# create shell split window
tmux new-window -n $TMUX_WIN2_NAME
tmux split-window -t $TMUX_SESS_NAME:$TMUX_WIN2_NAME -h
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN2_NAME.right "git status" Enter

# create gitlab-lib window
tmux new-window -n $TMUX_WIN3_NAME
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN3_NAME "cd ~/Code/rust-projects/rust-gitlab/ " Enter
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN3_NAME "vim -c CommandTBoot" Enter
tmux split-window -t $TMUX_SESS_NAME:$TMUX_WIN3_NAME -h
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN3_NAME "cd ~/Code/rust-projects/rust-gitlab/ " Enter

# create runners window
tmux new-window -n $TMUX_WIN4_NAME
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN4_NAME "cd ~/Code/gitlab-runners-local/ " Enter
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN4_NAME "vim -c CommandTBoot" Enter
tmux split-window -t $TMUX_SESS_NAME:$TMUX_WIN4_NAME -h
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN4_NAME "cd ~/Code/gitlab-runners-local/ " Enter

# create build docker window
tmux new-window -n $TMUX_WIN5_NAME
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN5_NAME "cd ~/Code/rust-projects/rust-build-docker/ " Enter
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN5_NAME "vim -c CommandTBoot" Enter
tmux split-window -t $TMUX_SESS_NAME:$TMUX_WIN5_NAME -h
tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN5_NAME "cd ~/Code/rust-projects/rust-build-docker/ " Enter


tmux attach -t $TMUX_SESS_NAME:$TMUX_WIN1_NAME.1

# tmux split-window -t $TMUX_SESS_NAME:$TMUX_WIN1_NAME -h
# tmux send-keys -t $TMUX_SESS_NAME:$TMUX_WIN1_NAME.right "git status" Enter
# tmux split-window -t $TMUX_SESS_NAME:$TMUX_WIN1_NAME.2 -v
