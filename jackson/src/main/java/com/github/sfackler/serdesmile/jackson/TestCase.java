package com.github.sfackler.serdesmile.jackson;

public final class TestCase<T> {
    public boolean rawBinary;
    public boolean sharedStrings;
    public boolean sharedProperties;
    public boolean writeEndMarker;
    public T value;
}
