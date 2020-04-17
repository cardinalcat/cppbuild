<h2>cargo like dependency manager for c++ based on pkg-config</h2>
<br/>
<br/>
<p>because of how pkg-config is used this program is able to use libraries installed with any package manager that distributed a .pc file</p>
<h4>Goals</h4>
<p>the intended base features of this program include: </p>
<ul>
<li>create projects</li>
<li>declaratively manage dependencies and verions</li>
<li>provide a tool for linting, automatic documentation, and code formatting</li>
<li>act as a package manager when the system package manager is unable to provided needed libraries</li>
</ul>
Currently working functions include basic setup, linking against external pkg-config libraries, and publishing cppbuild packages
