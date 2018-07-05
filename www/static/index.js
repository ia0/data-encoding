var wasm_promise = wasm_bindgen('./data_encoding_www_bg.wasm');
var load_promise = new Promise(function(resolve, reject) {
    window.onload = resolve;
});

Promise.all([wasm_promise, load_promise]).then(wasm_bindgen.init);

function createElement(name) { return document.createElement(name); }
function createTextNode(text) { return document.createTextNode(text); }
function appendChild(parent, child) { parent.appendChild(child); }
function insertBefore(parent, child, node) { parent.insertBefore(child, node); }
function removeChild(parent, child) { parent.removeChild(child); }
function setAttribute(node, name, value) { node.setAttribute(name, value); }
function removeAttribute(node, name) { node.removeAttribute(name); }
function getElementById(id) { return document.getElementById(id); }
function getElementByClass(node, name) {
    if (node.classList.contains(name)) return node;
    return node.getElementsByClassName(name)[0];
}
function value(node) { return node.value; }
function set_value(node, value) { node.value = value; }
function innerHTML(node) { return node.innerHTML; }
function set_innerHTML(node, value) { node.innerHTML = value; }
function addClass(node, name) { node.classList.add(name); }
function removeClass(node, name) { node.classList.remove(name); }
function setStorage(name, value) { localStorage.setItem(name, value); }
function getStorage(name) { return localStorage.getItem(name); }
function deleteStorage(name) { localStorage.removeItem(name); }
function clearStorage() { localStorage.clear(); }
function setHistory(name, value) {
    var url = new URL(document.location);
    url.searchParams.set(name, value);
    window.history.replaceState('', '', url.search);
}
function getHistory(name) {
    var value = (new URL(document.location)).searchParams.get(name);
    return value === null ? null : decodeURIComponent(value);
}
function deleteHistory(name) {
    var url = new URL(document.location);
    url.searchParams.delete(name);
    window.history.replaceState('', '', url.search);
}

function reset() {
    var url = new URL(document.location);
    localStorage.clear();
    window.history.replaceState('', '', '?');
    location.reload();
}
function get_id(event) {
    var encoding = event.target.closest('.s_encoding');
    return Number(encoding.id.match('^encoding_([0-9]*)$')[1]);
}
