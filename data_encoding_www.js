(function() {
    var wasm;
    const __exports = {};


    let cachedTextDecoder = new TextDecoder('utf-8');

    let cachegetUint8Memory = null;
    function getUint8Memory() {
        if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
            cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
        }
        return cachegetUint8Memory;
    }

    function getStringFromWasm(ptr, len) {
        return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
    }

    const heap = new Array(32);

    heap.fill(undefined);

    heap.push(undefined, null, true, false);

    let heap_next = heap.length;

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

    __exports.__wbg_createElement_2bca55cb0f1d8aa6 = function(arg0, arg1) {
        let varg0 = getStringFromWasm(arg0, arg1);
        return addHeapObject(createElement(varg0));
    };

    __exports.__wbg_createTextNode_f7acab4be4031136 = function(arg0, arg1) {
        let varg0 = getStringFromWasm(arg0, arg1);
        return addHeapObject(createTextNode(varg0));
    };

function getObject(idx) { return heap[idx]; }

__exports.__wbg_appendChild_2a549c68d1a623ae = function(arg0, arg1) {
    appendChild(getObject(arg0), getObject(arg1));
};

__exports.__wbg_insertBefore_1c1c38ac08e31886 = function(arg0, arg1, arg2) {
    insertBefore(getObject(arg0), getObject(arg1), getObject(arg2));
};

__exports.__wbg_removeChild_d6d9e46dcbfc7238 = function(arg0, arg1) {
    removeChild(getObject(arg0), getObject(arg1));
};

__exports.__wbg_setAttribute_d3dad635dc2f8b77 = function(arg0, arg1, arg2, arg3, arg4) {
    let varg1 = getStringFromWasm(arg1, arg2);
    let varg3 = getStringFromWasm(arg3, arg4);
    setAttribute(getObject(arg0), varg1, varg3);
};

__exports.__wbg_removeAttribute_abc76d1221984ae6 = function(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    removeAttribute(getObject(arg0), varg1);
};

__exports.__wbg_getElementById_afe5fae05621a5c0 = function(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    return addHeapObject(getElementById(varg0));
};

__exports.__wbg_getElementByClass_8c81e052dbdcaf8e = function(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    return addHeapObject(getElementByClass(getObject(arg0), varg1));
};

let cachedTextEncoder = new TextEncoder('utf-8');

let WASM_VECTOR_LEN = 0;

function passStringToWasm(arg) {

    const buf = cachedTextEncoder.encode(arg);
    const ptr = wasm.__wbindgen_malloc(buf.length);
    getUint8Memory().set(buf, ptr);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
}

let cachegetUint32Memory = null;
function getUint32Memory() {
    if (cachegetUint32Memory === null || cachegetUint32Memory.buffer !== wasm.memory.buffer) {
        cachegetUint32Memory = new Uint32Array(wasm.memory.buffer);
    }
    return cachegetUint32Memory;
}

__exports.__wbg_value_e89469098ccd35b9 = function(ret, arg0) {

    const retptr = passStringToWasm(value(getObject(arg0)));
    const retlen = WASM_VECTOR_LEN;
    const mem = getUint32Memory();
    mem[ret / 4] = retptr;
    mem[ret / 4 + 1] = retlen;

};

__exports.__wbg_setvalue_3ae4c1d24e58c6e8 = function(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    set_value(getObject(arg0), varg1);
};

__exports.__wbg_innerHTML_fbc37cd148a71f2e = function(ret, arg0) {

    const retptr = passStringToWasm(innerHTML(getObject(arg0)));
    const retlen = WASM_VECTOR_LEN;
    const mem = getUint32Memory();
    mem[ret / 4] = retptr;
    mem[ret / 4 + 1] = retlen;

};

__exports.__wbg_setinnerHTML_4c0c636f534bd6cc = function(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    set_innerHTML(getObject(arg0), varg1);
};

__exports.__wbg_focus_7e26f5ab08bd68e6 = function(arg0) {
    focus(getObject(arg0));
};

__exports.__wbg_addClass_719db2a08a755b86 = function(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    addClass(getObject(arg0), varg1);
};

__exports.__wbg_removeClass_c18feb5580f55a12 = function(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    removeClass(getObject(arg0), varg1);
};

__exports.__wbg_hasClass_aec1e2843aa88262 = function(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    return hasClass(getObject(arg0), varg1);
};

__exports.__wbg_setStorage_615e46b367bb2a3d = function(arg0, arg1, arg2, arg3) {
    let varg0 = getStringFromWasm(arg0, arg1);
    let varg2 = getStringFromWasm(arg2, arg3);
    setStorage(varg0, varg2);
};

__exports.__wbg_getStorage_bed7f7a118a53328 = function(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    return addHeapObject(getStorage(varg0));
};

__exports.__wbg_deleteStorage_afc355584701884f = function(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    deleteStorage(varg0);
};

__exports.__wbg_setHistory_c1c8e303f2ca73ea = function(arg0, arg1, arg2, arg3) {
    let varg0 = getStringFromWasm(arg0, arg1);
    let varg2 = getStringFromWasm(arg2, arg3);
    setHistory(varg0, varg2);
};

__exports.__wbg_getHistory_bbdb2019bc595316 = function(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    return addHeapObject(getHistory(varg0));
};

__exports.__wbg_deleteHistory_fceb23cc0d7506d3 = function(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    deleteHistory(varg0);
};
/**
* @returns {void}
*/
__exports.init = function() {
    return wasm.init();
};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.swap_left = function(arg0) {
    return wasm.swap_left(arg0);
};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.swap_right = function(arg0) {
    return wasm.swap_right(arg0);
};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.delete_encoding = function(arg0) {
    return wasm.delete_encoding(arg0);
};

/**
* @returns {void}
*/
__exports.add_encoding = function() {
    return wasm.add_encoding();
};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.toggle_bit_order = function(arg0) {
    return wasm.toggle_bit_order(arg0);
};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.toggle_trailing_bits = function(arg0) {
    return wasm.toggle_trailing_bits(arg0);
};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.text_update = function(arg0) {
    return wasm.text_update(arg0);
};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.spec_update = function(arg0) {
    return wasm.spec_update(arg0);
};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.load_preset = function(arg0) {
    return wasm.load_preset(arg0);
};

/**
* @param {string} arg0
* @returns {void}
*/
__exports.toggle_menu = function(arg0) {
    const ptr0 = passStringToWasm(arg0);
    const len0 = WASM_VECTOR_LEN;
    try {
        return wasm.toggle_menu(ptr0, len0);

    } finally {
        wasm.__wbindgen_free(ptr0, len0 * 1);

    }

};

/**
* @param {string} arg0
* @returns {void}
*/
__exports.goto_tutorial = function(arg0) {
    const ptr0 = passStringToWasm(arg0);
    const len0 = WASM_VECTOR_LEN;
    try {
        return wasm.goto_tutorial(ptr0, len0);

    } finally {
        wasm.__wbindgen_free(ptr0, len0 * 1);

    }

};

/**
* @param {number} arg0
* @returns {void}
*/
__exports.move_focus = function(arg0) {
    return wasm.move_focus(arg0);
};

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

__exports.__wbindgen_object_drop_ref = function(i) { dropObject(i); };

__exports.__wbindgen_is_null = function(idx) {
    return getObject(idx) === null ? 1 : 0;
};

__exports.__wbindgen_string_get = function(i, len_ptr) {
    let obj = getObject(i);
    if (typeof(obj) !== 'string') return 0;
    const ptr = passStringToWasm(obj);
    getUint32Memory()[len_ptr / 4] = WASM_VECTOR_LEN;
    return ptr;
};

function init(path_or_module) {
    let instantiation;
    const imports = { './data_encoding_www': __exports };
    if (path_or_module instanceof WebAssembly.Module) {
        instantiation = WebAssembly.instantiate(path_or_module, imports)
        .then(instance => {
        return { instance, module: path_or_module }
    });
} else {
    const data = fetch(path_or_module);
    if (typeof WebAssembly.instantiateStreaming === 'function') {
        instantiation = WebAssembly.instantiateStreaming(data, imports);
    } else {
        instantiation = data
        .then(response => response.arrayBuffer())
        .then(buffer => WebAssembly.instantiate(buffer, imports));
    }
}
return instantiation.then(({instance}) => {
    wasm = init.wasm = instance.exports;

});
};
self.wasm_bindgen = Object.assign(init, __exports);
})();
