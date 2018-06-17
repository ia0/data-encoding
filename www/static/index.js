var wasm_promise = wasm_bindgen('./data_encoding_www_bg.wasm');
var load_promise = new Promise(function(resolve, reject) {
    window.onload = resolve;
});

Promise.all([wasm_promise, load_promise]).then(wasm_bindgen.init);

function body() { return document.body; }
function createElement(name) { return document.createElement(name); }
function createTextNode(text) { return document.createTextNode(text); }
function appendChild(parent, child) { parent.appendChild(child); }
function setAttribute(node, name, value) { node.setAttribute(name, value); }
function getElementById(id) { return document.getElementById(id); }
function value(node) { return node.value; }
function set_value(node, value) { node.value = value; }
function addClass(node, name) { node.classList.add(name); }
function removeClass(node, name) { node.classList.remove(name); }
function is_checked(node) { return node.checked; }
function set_checked(node) { node.checked = true; }
