# 逃离塔科夫物品管理器AI编程指导文件

1. 未经明确许可，禁止修订本文件
2. 文件应以UTF-8编码进行存储和处理
3. 系统为中文系统，注意处理GBK乱码

## 前端

1. 采用vue3架构
2. UI库使用Vuetify
3. 主界面显示各设施卡片，如有等级，则有选框
4. 底部展示所需材料，允许筛选带勾

## 后端

1. 采用rust语言

## 静态数据集

1. 静态数据集位于dataset目录
2. [items](./dataset/items.json)存储了所有物品id、物品官方英文名
3. [items_cn](./dataset/items.cn.json)存储了所有物品id、物品官方中文名
4. [facilities](./dataset/facilities.json)存储了藏身处各设施id、官方英文名、各设施的前置设施id
5. [facilities_cn](./dataset/facilities.cn.json)存储了藏身处各设施id、官方中文名
6. [hideout](./dataset/hideout.json)存储了藏身处各设施的id、各等级升级所需物品id与个数、是否带勾

## 数据库

1. 优先采用本地SQLite
2. 允许用户输入sql连接字符串，如PostgreSQL、MySQL等等

## 用户注册

1. 因为存在数据库，需要有用户注册与登录界面

## 发布

1. 软件最后将发布至DockerHub
