export function isGenerator(obj) {
    if (!obj)
        return false;
    if (!obj.constructor)
        return false;
    if (!obj.constructor.constructor)
        return false;
    if (!obj.constructor.constructor.name)
        return false;
    return (obj.constructor.constructor.name == 'GeneratorFunction' ||
        obj.constructor.constructor.name == 'AsyncGeneratorFunction');
}
//# sourceMappingURL=Utils.js.map