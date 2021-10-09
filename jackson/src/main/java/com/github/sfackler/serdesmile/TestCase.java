package com.github.sfackler.serdesmile;

public final class TestCase<T> {
    public boolean rawBinary;
    public boolean sharedStrings;
    public boolean sharedProperties;
    public boolean writeEndMarker;
    public T value;
}
