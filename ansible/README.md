# Empowerd Ansible installer for Raspiberry-Pi
## Directory structure
* "packages": Place grafana and empowerd debian packages here
* "pi-gen": Patches for Raspi image creation

## Installation
1. Clone pi-gen repository and apply patches from pi-gen directory.
1. Build raspi image and write it to raspi HDD.
1. The pi will fetch its IP via DHCP by default.
1. Login as user pi:password via SSH
1. Set secure password: `passwd`
1. Setup passwordless SSH login:
  * Generate key: `ssh-keygen -b 4096 .ssh/id_rsa_pi`
  * Copy public key to pi: `scp .ssh/id_rsa_pi.pub 192.168.1.9:.ssh/authorized_keys`
1. Temporary replace the raspi IP address in "hosts.yml" with the IP from DHCP.
1. Configure network via Ansible: `ansible-playbook playbook.yml -l empowerd-pi --tags network_interfaces`
1. Reboot
1. Set secure passwords and add certificates in all Ansible files with "TODO" sections.
   Then encrypt them with `ansible-vault`
1. Run Ansible: `ansible-playbook playbook.yml -l empowerd-pi`

## Backups
1. The pi creates a database backup every day at "/root/influx.bak" which should be stored externally.
1. To restore the backup, run `influxd restore -portable /root/influx.bak`.
