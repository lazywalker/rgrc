#!/usr/bin/env fish
#
# To load in ~/.config/fish/fish.conf or a new file in
# ~/.config/fish/conf.d add:
# source /etc/grc.fish (path may depend on install method)
#
# See also the plugin at https://github.com/oh-my-fish/plugin-grc

set -U grc_plugin_execs cat cvs df diff dig gcc g++ ls ifconfig \
       make mount mtr netstat ping ps head tail traceroute \
       wdiff blkid du dnf docker docker-compose docker-machine env id ip iostat journalctl kubectl \
       last lsattr lsblk lspci lsmod lsof lsusb getfacl getsebool ulimit uptime nmap \
       fdisk findmnt free semanage sar ss sysctl systemctl stat showmount \
       tcpdump tune2fs vmstat w who sockstat

for executable in $grc_plugin_execs
    if type -q $executable
        function $executable --inherit-variable executable --wraps=$executable
            if isatty 1
                rgrc $executable $argv
            else
                eval command $executable $argv
            end
        end
    end
end
