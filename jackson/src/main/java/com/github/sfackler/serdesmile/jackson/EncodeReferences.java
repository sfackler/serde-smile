package com.github.sfackler.serdesmile.jackson;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.dataformat.smile.SmileFactory;
import com.fasterxml.jackson.dataformat.smile.SmileGenerator.Feature;
import com.fasterxml.jackson.dataformat.smile.databind.SmileMapper;
import com.github.sfackler.serdesmile.jackson.TestCase;

public final class EncodeReferences {
    private static ObjectMapper JSON_MAPPER = new ObjectMapper();

    public static void main(String[] args) throws IOException {
        processTestCases("integer", new TypeReference<TestCase<Integer>>() {
        });
        processTestCases("long", new TypeReference<TestCase<Long>>() {
        });
        processTestCases("string", new TypeReference<TestCase<String>>() {
        });
        processTestCases("float", new TypeReference<TestCase<Float>>() {
        });
        processTestCases("double", new TypeReference<TestCase<Double>>() {
        });
        processTestCases("boolean", new TypeReference<TestCase<Boolean>>() {
        });
        processTestCases("binary", new TypeReference<TestCase<byte[]>>() {
        });
    }

    private static <T> void processTestCases(String category, TypeReference<TestCase<T>> type) throws IOException {
        List<Path> files;
        try (Stream<Path> stream = Files.list(Paths.get("../tests", category))) {
            files = stream.filter(p -> p.getFileName().toString().endsWith(".json")).collect(Collectors.toList());
        }

        for (Path file : files) {
            processTestCase(file, type);
        }
    }

    private static <T> void processTestCase(Path path, TypeReference<TestCase<T>> type) throws IOException {
        TestCase<T> testCase = JSON_MAPPER.readValue(path.toFile(), type);

        SmileFactory factory = new SmileFactory();
        factory.configure(Feature.ENCODE_BINARY_AS_7BIT, !testCase.rawBinary);
        factory.configure(Feature.CHECK_SHARED_STRING_VALUES, testCase.sharedStrings);
        factory.configure(Feature.CHECK_SHARED_NAMES, testCase.sharedProperties);
        factory.configure(Feature.WRITE_END_MARKER, testCase.writeEndMarker);
        SmileMapper smileMapper = new SmileMapper(factory);

        String filename = path.getFileName().toString().replaceAll("\\.json$", ".smile");
        Path outPath = path.getParent().resolve(filename);

        smileMapper.writeValue(outPath.toFile(), testCase.value);
    }
}
