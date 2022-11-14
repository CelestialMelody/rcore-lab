# 实验环境配置

> http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html

**补充：manjaro docker**

```shell
yay -S docker

sudo mkdir /etc/docker

sudo vim /etc/docker/daemon.json 

# 开机启动
sudo systemctl enable docker.service

# 关闭开机启动
sudo systemctl disable docker.service

# 启动
sudo systemctl start docker.service

sudo systemctl restart docker.service

# 把工作用户加入 docker 组，避免使用 root 帐号工作 -> 守护进程问题
sudo gpasswd -a $USER docker

# 重新加载配置
systemctl daemon-reload
systemctl restart docker

# 重启
reboot
```

```shell
git clone https://github.com/LearningOS/rust-based-os-comp2022.git

# 配置环境
make build_docker

# 进入
make docker
```

```shell
# rust 安装
curl https://sh.rustup.rs -sSf | sh
```

