import os
from pathlib import Path
from typing import Any

import numpy as np
from keras.layers import Add, Input
from keras.models import Model as KerasModel
from keras.models import load_model

from src.cnn.train import fen_from_onehot, process_image, process_image_bytes, weighted_categorical_crossentropy

SQUARE_SIZE = 40


def get_models_names() -> list[str]:
    base_path = Path() / "models"

    names = os.listdir(base_path / "custom") or os.listdir(base_path / "default")

    assert names, "There are not models files (.hdf5)"
    return names


def _load_all_models(names: list[str]) -> list[Any]:
    models = []
    # TODO: TMP
    path_to_models = "./models/default/"
    for model_name in names:
        models.append(
            load_model(
                f"{path_to_models}{model_name}",
                custom_objects={
                    "loss": weighted_categorical_crossentropy(
                        np.array(
                            [
                                1 / (0.30 * 4),
                                1 / (0.20 * 4),
                                1 / (0.20 * 4),
                                1 / (0.20 * 4),
                                1 / 1,
                                1 / (0.10 * 4),
                                1 / (0.30 * 4),
                                1 / (0.20 * 4),
                                1 / (0.20 * 4),
                                1 / (0.20 * 4),
                                1 / 1,
                                1 / (0.10 * 4),
                                1 / (64 - 10),
                            ]
                        )
                    )
                },
            )
        )
    return models


def _get_stacked_model(models):
    input_layer = Input(shape=(SQUARE_SIZE, SQUARE_SIZE, 3))
    xs = [model(input_layer) for model in models]
    out = Add()(xs)

    return KerasModel(inputs=[input_layer], outputs=out)


def init_model() -> KerasModel:
    models_names = get_models_names()
    keras_models = _load_all_models(models_names)
    model = _get_stacked_model(keras_models)
    return model


def predict_fen_from_image(image_array: np.ndarray, model: KerasModel) -> str:
    processed_image = process_image_bytes(image_array)
    pred = model.predict(processed_image, verbose=0).argmax(axis=1).reshape(-1, 8, 8)
    fen = fen_from_onehot(pred[0])

    return fen.replace("-", "/")
