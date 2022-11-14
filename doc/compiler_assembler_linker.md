[编译流程](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E5%85%A8%E5%B1%80%E6%95%B0%E6%8D%AE%E6%AE%B5%E4%B8%AD%E3%80%82-,%E7%BC%96%E8%AF%91%E6%B5%81%E7%A8%8B,-%23)

链接器所做的事情

- 将来自不同目标文件的段在目标内存布局中[重新排布](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E7%AC%AC%E4%B8%80%E4%BB%B6%E4%BA%8B%E6%83%85%E6%98%AF%E5%B0%86%E6%9D%A5%E8%87%AA%E4%B8%8D%E5%90%8C%E7%9B%AE%E6%A0%87%E6%96%87%E4%BB%B6%E7%9A%84%E6%AE%B5%E5%9C%A8%E7%9B%AE%E6%A0%87%E5%86%85%E5%AD%98%E5%B8%83%E5%B1%80%E4%B8%AD%E9%87%8D%E6%96%B0%E6%8E%92%E5%B8%83)
- 将符号[替换](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E9%87%8D%E6%96%B0%E6%8E%92%E5%B8%83-,%E7%AC%AC%E4%BA%8C%E4%BB%B6%E4%BA%8B%E6%83%85%E6%98%AF%E5%B0%86%E7%AC%A6%E5%8F%B7%E6%9B%BF%E6%8D%A2%E4%B8%BA%E5%85%B7%E4%BD%93%E5%9C%B0%E5%9D%80%E3%80%82,-%E8%BF%99%E9%87%8C%E7%9A%84%E7%AC%A6%E5%8F%B7)为具体地址



[调整内核的内存布局](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/4first-instruction-in-kernel2.html#:~:text=%E5%BA%93%20core%20%E4%B8%AD%E3%80%82-,%E8%B0%83%E6%95%B4%E5%86%85%E6%A0%B8%E7%9A%84%E5%86%85%E5%AD%98%E5%B8%83%E5%B1%80%23,-%E7%94%B1%E4%BA%8E%E9%93%BE%E6%8E%A5)

由于链接器默认的内存布局并不能符合我们的要求，实现与 Qemu 的正确对接，我们可以通过 **链接脚本** (Linker Script) 调整链接器的行为，使得最终生成的可执行文件的内存布局符合我们的预期。

> [链接脚本(Linker Scripts)语法和规则解析](https://blog.csdn.net/m0_47799526/article/details/108765403)
>
> [视频教程](https://www.bilibili.com/video/BV1gL411A7dX/?spm_id_from=333.788&vd_source=fff8a96619bd3da6d1cb5d5c1ede2cf1)



[寄存器保存与编译器优化](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E6%B3%A8%E8%A7%A3-,%E5%AF%84%E5%AD%98%E5%99%A8%E4%BF%9D%E5%AD%98%E4%B8%8E%E7%BC%96%E8%AF%91%E5%99%A8%E4%BC%98%E5%8C%96,-%E8%BF%99%E9%87%8C%E5%80%BC%E5%BE%97%E8%AF%B4%E6%98%8E)



.S文件，会进行预处理、汇编等操作。

.s文件，在后期阶段不在进行预处理操作，只有汇编操作。

.asm文件，等同于.s文件。因为汇编本质上是纯文本的，不管用什么后缀都可以。所以一般dos和windows下以.asm为主，linux下以.s为主。



---

其他工具

> https://www.runoob.com/linux/linux-comm-xargs.html
