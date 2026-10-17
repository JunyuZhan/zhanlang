---
title: zhanlang（詹语言）— AI 写实现，共同验证意图
---

<div class="hero">
<p class="eyebrow">面向 vibe coding 的编程语言</p>
<h1>AI 写实现，人和编译器共同验证意图。</h1>
<p class="sub">每个对外可见的函数必须携带 <code>spec</code>。<code>spec</code> 是唯一真相源——意图本身也是被验证的对象。</p>
<div class="cta">
<a class="btn btn-primary" href="../play/">打开 Playground</a>
<a class="btn btn-ghost" href="../spec/">阅读规格书</a>
</div>
</div>

<div class="section">
<p class="eyebrow">核心不变量</p>
<h2>三条规则，编译期强制</h2>
<div class="card-grid">
<div class="card">
<h3>可见函数必须有 spec</h3>
<p>每个对外可见的函数在定义处声明前置条件、后置条件与样例；没有 spec 的公开函数无法通过编译。</p>
</div>
<div class="card">
<h3>spec 是唯一真相源</h3>
<p>文档、注释、测试都不再各自为政——需求就写在代码旁边，并且可被执行。</p>
</div>
<div class="card">
<h3>spec 本身也被验证</h3>
<p>编译器同时检查：实现满足 spec，且 spec 非空洞、不矛盾、样例可达。</p>
</div>
</div>
</div>

<div class="section">
<p class="eyebrow">示例</p>
<h2>意图写进代码</h2>
<pre><code class="language-zhanlang">@@EXAMPLE@@</code></pre>
<p style="margin-top:16px"><a href="../play/">在 Playground 中继续编辑 →</a></p>
</div>

<div class="section">
<p class="eyebrow">路线图</p>
<h2>状态：M1–M5 全部完成</h2>
<ul class="roadmap">
<li>M1 解析与类型检查</li>
<li>M2 spec 自检</li>
<li>M3 运行时断言</li>
<li>M4 最小静态验证</li>
<li>M5 状态词汇</li>
</ul>
</div>
