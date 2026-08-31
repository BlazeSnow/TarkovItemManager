# Tarkov Item Manager AI编程指导文件

1. 未经明确许可，禁止修订本文件
2. 文件应以UTF-8编码进行存储和处理
3. 系统为中文系统，注意处理GBK乱码

## 前端

1. 采用vue3架构
2. UI库使用Vuetify
3. 主界面显示各设施卡片，如有等级，则选择当前已有等级
4. 底部展示从当前等级升级至满级所需材料，允许按是否需要战局中带出（带勾，Found in Raid）筛选

## 后端

1. 采用rust语言

## 静态数据集

1. 静态数据集位于dataset目录
2. items.json存储所有物品的数值ID和官方中文名
3. facilities.json存储藏身处设施的数值ID和官方中文名
4. merchants.json存储商人的数值ID和名称
5. 根 hideout.json 存储 PVE 快照元数据和有序 upgradeFiles 清单；dataset/hideout/<设施ID>.json 分别存储该设施各等级升级记录：材料物品ID、个数、是否需要战局中带出（带勾，Found in Raid）、设施/商人/技能/任务/版本包前置条件和建造时间
6. 静态数据当前采用扁平数值ID契约；后端加载器适配应作为独立改造，不得混入数据整理工作

## 数据库

1. 优先采用本地SQLite
2. 允许用户输入sql连接字符串，如PostgreSQL、MySQL等等

## 用户注册

1. 因为存在数据库，需要有用户注册与登录界面

## 发布

1. 软件使用GitHub Action发布
