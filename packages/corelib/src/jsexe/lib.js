function ensure_object(obj, prop) {
    if (!is_object(obj[prop])) {
        obj[prop] = {};
    }
}
function ensure_array(obj, prop) {
    if (!is_array(obj[prop])) {
        obj[prop] = [];
    }
}
function is_object(obj) { return obj && (typeof obj === "object") && !Array.isArray(obj) }
function is_array(obj) { return obj && Array.isArray(obj) }
