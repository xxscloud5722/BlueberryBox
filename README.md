# Blueberry Box (NodeJS 服务端渲染) 
- 使用Vue 或者 React 单页面提供后端服务
- 提供 `SEO` 优化配置

# 项目说明
- `app.js` 项目主程序
- `seo.js` SEO配置文件


# 安装项目
```shell
npm install
# 将单页面项目放入到根目录的web目录
# 将seo.json 配置文件也放入根目录web目录

# 默认启动, 不缓存文件信息; 动态更新文件
node app.js

# 缓存目录文件信息性能搞一点点 ; 不可以动态更新配置
node app.js --cache

```

# SEO 配置说明
- `regexp` 正则匹配
- `preMatch` 路径前缀匹配
- `tailMatch` 路径后缀匹配
- `head`  替换的头部元素
```JSON
[
  {
    "regexp": ".",
    "head": [
      {
        "name": "mobile-web-app-capable",
        "content": "yes"
      },
      {
        "name": "description",
        "content": "京东JD.COM-专业的综合网上购物商城，为您提供正品低价的购物选择、优质便捷的服务体验。商品来自全球数十万品牌商家，囊括家电、手机、电脑、服装、居家、母婴、美妆、个护、食品、生鲜等丰富品类，满足各种购物需求。"
      }
    ]
  },
  {
    "regexp": "",
    "preMatch": "/login",
    "tailMatch": "",
    "head": [
      {
        "name": "login",
        "content": "login"
      },
      {
        "name": "description",
        "content": "我这里是京东的登录"
      }
    ]
  }
]
```