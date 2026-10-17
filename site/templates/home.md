---
title: zhanlang — AI writes, we verify intent
---

<div class="hero">
<p class="eyebrow">A programming language for vibe coding</p>
<h1>AI writes the implementation, humans and the compiler verify intent.</h1>
<p class="sub">Every publicly visible function must carry a <code>spec</code>. The <code>spec</code> is the single source of truth — and the intent itself is verified too.</p>
<div class="cta">
<a class="btn btn-primary" href="play/">Open the Playground</a>
<a class="btn btn-ghost" href="spec/">Read the spec</a>
</div>
</div>

<div class="section">
<p class="eyebrow">Core invariant</p>
<h2>Three rules, enforced at compile time</h2>
<div class="card-grid">
<div class="card">
<h3>Visible functions require a spec</h3>
<p>Every publicly visible function declares its preconditions, postconditions, and examples where it is defined; a public function without a spec does not compile.</p>
</div>
<div class="card">
<h3>The spec is the single source of truth</h3>
<p>Docs, comments, and tests stop drifting apart — requirements live next to the code and are executable.</p>
</div>
<div class="card">
<h3>The spec itself is verified</h3>
<p>The compiler checks both directions: the implementation satisfies the spec, and the spec is non-vacuous, consistent, and reachable.</p>
</div>
</div>
</div>

<div class="section">
<p class="eyebrow">Example</p>
<h2>Intent, written into the code</h2>
<pre><code class="language-zhanlang">@@EXAMPLE@@</code></pre>
<p style="margin-top:16px"><a href="play/">Keep editing in the Playground →</a></p>
</div>

<div class="section">
<p class="eyebrow">Roadmap</p>
<h2>Status: M1–M5 complete</h2>
<ul class="roadmap">
<li>M1 Parsing & type checking</li>
<li>M2 Spec self-check</li>
<li>M3 Runtime assertions</li>
<li>M4 Minimal static verification</li>
<li>M5 State vocabulary</li>
</ul>
</div>
