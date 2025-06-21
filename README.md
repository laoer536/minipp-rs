# minipp

Quickly identify unused files in your project to help slim down your codebase.

It's Rust version.

You can see nodejs version at [https://github.com/laoer536/minipp](https://github.com/laoer536/minipp)

## Performance：

For such a front-end project, there are `9843` files under the src folder:

![image.png](https://s2.loli.net/2025/06/21/WzyNGFKScLdBP62.png)

### Rust-minipp 269ms

![image.png](https://s2.loli.net/2025/06/21/3rFtOCib2nagqvS.png)

### Nodejs-minipp 2.851s

![image.png](https://s2.loli.net/2025/06/21/3xcpfM2mVKREIUY.png)

> **Rust has a performance improvement of about 10 times compared to Nodejs! 🤪**